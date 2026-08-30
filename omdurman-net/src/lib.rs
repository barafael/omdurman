use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use chrono::{DateTime, Utc};
use matchbox_socket::RtcIceServerConfig;
use omdurman_rules::MovementPoints;
use omdurman_rules::OptionalRule;
use omdurman_rules::effects::GameEffect;
use omdurman_types::{HexCoord, Player, Scenario, SpriteRef};
use serde::{Deserialize, Serialize};

/// Shared OpenAI-compatible LLM transport (config + `request_completion`).
/// Reused by `omdurman-app` (flavour text) and `omdurman-bot` (strategy advisor).
pub mod llm;

pub const SIGNALING_SERVER: &str = if let Some(s) = option_env!("MATCHBOX_SERVER") {
    s
} else {
    "wss://omdurman-matchbox.fly.dev"
};

// -- Event-sourced game record ---------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitialGameState {
    pub seed: u64,
}

/// A game-state mutation. These are the only `NetMsg` payloads that get
/// recorded into [`GameRecord`] and replayed for late joiners. Adding a
/// variant here automatically participates in recording and replay.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, strum::IntoStaticStr)]
pub enum GameEvent {
    /// Host-committed faction assignment that starts the game. Maps each
    /// player's `PeerId` (as its string form, stable within the session) to
    /// the `Player` (faction) they will command. Recorded + replayed, so a
    /// late joiner learns the bindings via the snapshot path.
    StartGame {
        assignments: Vec<(PeerId, Player)>,
        /// The scenario the host committed to. Selects which board loads
        /// (`Campaign` -> campaign map, otherwise the Fall-of-Khartoum map) and
        /// seeds the rules engine's turn track. Recorded + replayed so late
        /// joiners and history replay agree on both.
        #[serde(default)]
        scenario: Scenario,
        /// Optional rule selected by the Dervish host for a campaign game
        /// (§10.11 RiverMines, §10.21 RiverChain). `None` if no optional rule
        /// or the scenario doesn't support them.
        #[serde(default)]
        optional_rule: Option<OptionalRule>,
    },
    /// A semantic game action resolved by the rule engine (§effect system).
    Effect(GameEffect),
    PlaceUnit {
        sprite: SpriteRef,
        #[serde(default)]
        coord: HexCoord,
        is_boat: bool,
    },
    /// Remove a unit from the board during setup (§9) so it can be
    /// re-placed.  Only legal during Phase::Setup.  Recorded + replayed
    /// like PlaceUnit.
    RemoveUnit { sprite: SpriteRef },
    MoveUnit {
        sprite: SpriteRef,
        to_q: i32,
        to_r: i32,
        #[serde(default)]
        cost: MovementPoints,
        /// The hexes entered, excluding the start and ending at the destination
        /// (the picker's BFS route). Lets the rules engine cost the move by
        /// terrain (§5.11), classify gunboat up/downstream steps (§5.24), and
        /// enforce the ZOC-stop rule per hex along the route (§5.26/§5.43).
        /// Empty on legacy records / direct destination-only moves, in which
        /// case the engine falls back to the supplied `cost`.
        #[serde(default)]
        path: Vec<HexCoord>,
    },
}

/// One entry in the canonical event log: a `GameEvent` plus the metadata
/// every peer needs to replay it deterministically.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordedEvent {
    pub utc: DateTime<Utc>,
    #[serde(default)]
    pub sender_idx: Option<u8>,
    /// Canonical, host-assigned global sequence number. Identical on every
    /// peer for the same event, so all peers' logs are byte-for-byte ordered
    /// the same way (§ordering).
    pub seq: u32,
    /// Submission identity of the event (see [`NetMsg::Game`]). `None` for
    /// records written before uids existed; identity dedup and confirmation
    /// simply do not engage for those.
    #[serde(default)]
    pub uid: Option<u64>,
    pub payload: GameEvent,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameRecord {
    pub initial_state: InitialGameState,
    pub events: Vec<RecordedEvent>,
}

/// Display-only state shared between peers but never recorded -- cursors,
/// identity, transient UI selections. Sent on the unreliable channel
/// (except `PlayerInfo`, which is one-shot on connect via reliable).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Ephemeral {
    CursorPos {
        pos: [f32; 2],
    },
    PlayerInfo {
        name: String,
        color: [u8; 3],
    },
    EventViewerSelect(i32),
    /// Lobby faction pick (live preview). `None` = undecided. The authoritative
    /// binding is committed by the host via [`GameEvent::StartGame`].
    FactionChoice(Option<Player>),
    /// Lobby scenario pick (live preview, host-authoritative). The committed
    /// value travels in [`GameEvent::StartGame`].
    ScenarioChoice(Scenario),
    /// Lobby spectator toggle (live preview). A spectator joins the game to
    /// watch only: it is never placed in the authoritative faction binding
    /// (`StartGame` assignments), so all action gates no-op for it. Kept
    /// separate from `FactionChoice` so peers can distinguish "spectating" from
    /// "undecided" in the lobby roster.
    SpectatorChoice(bool),
}

/// Snapshot-handshake messages. Always reliable.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Control {
    RequestSnapshot,
    SnapshotReceived,
    GameHistory(GameRecord),
}

// -- Wire protocol ---------------------------------------------------------

/// Top-level wire envelope. The sub-enums encode the *intent* of a message --
/// game-mutating vs ephemeral vs control -- so receivers can route each
/// category without an exhaustive top-level match.
///
/// Game events use a host-relay protocol to guarantee a single global order
/// (§ordering): a non-host peer submits its event as [`NetMsg::Game`] to the
/// host only; the host assigns the next canonical sequence number and
/// rebroadcasts it as [`NetMsg::Sequenced`] to every peer (including looping
/// it back to itself). *Every* peer -- originator included -- applies and
/// records a game event only when it arrives as `Sequenced`, so all peers
/// observe the identical ordered stream.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetMsg {
    /// Unsequenced game-event submission, sent guest->host. Carries a
    /// submission-unique `uid` (see `PendingEdits::submit_game`) so the
    /// author can confirm sequencing via the echo, and the host can dedupe
    /// retransmissions idempotently instead of sequencing an event twice.
    /// The host orders it and rebroadcasts as [`NetMsg::Sequenced`]; it is
    /// never applied directly.
    Game {
        uid: u64,
        event: GameEvent,
    },
    /// Canonical, host-sequenced game event, sent host->all. This is the only
    /// form that is applied to the world and appended to the event log. The
    /// `uid` is the submission identity of the enclosed event.
    Sequenced {
        seq: u32,
        uid: u64,
        event: GameEvent,
    },
    Ephemeral(Ephemeral),
    Control(Control),
}

/// Encode a `NetMsg` for the wire. Returns `None` if encoding fails or would
/// produce a zero-length payload.
///
/// WebRTC data channels may *silently* drop a zero-byte payload -- `try_send`
/// returns `Ok` but the message never fires `onmessage` on the receiver, so the
/// loss is invisible on both ends (the receiver never calls [`decode`], so even
/// our decode-error `warn!` never fires). A real `NetMsg` always encodes to >=1
/// byte (the enum variant tag), so the only way to hit the empty case is a
/// postcard failure (OOM); we surface `None` so callers skip the send entirely
/// rather than putting an empty packet on the wire and hoping it lands.
pub fn enc_msg(msg: &NetMsg) -> Option<Box<[u8]>> {
    match postcard::to_allocvec(msg) {
        Ok(v) if !v.is_empty() => Some(v.into_boxed_slice()),
        Ok(_) => {
            error!(
                "postcard produced an empty NetMsg encoding; dropping (would be silently lost on WebRTC)"
            );
            None
        }
        Err(e) => {
            error!("postcard encode failed: {e}");
            None
        }
    }
}

pub fn decode(raw: &[u8]) -> Option<NetMsg> {
    postcard::from_bytes(raw)
        .inspect_err(|e| warn!("matchbox decode error: {e}"))
        .ok()
}

/// Minimum time (accumulated over frames) that the peer set and our own id
/// must have been unchanged before the host is allowed to sequence events.
/// See [`NetState::election_stable_secs`].
pub const SEQ_STABILIZE_SECS: f32 = 1.0;

/// A rejoining peer's record is wiped by `handle_reconnect`; it must install
/// a canonical history before resuming host authority (otherwise a
/// re-elected host spins a rogue line off its wiped record while a superior
/// line lives on the guests). If nobody serves a history within this budget,
/// the room is dead anyway and it may bootstrap on the wiped record.
pub const RESYNC_BOOTSTRAP_SECS: f32 = 15.0;

/// Bounded ring of recently applied submission uids. Large enough to cover
/// every uid that could still be re-delivered (retransmit retries and echoes
/// are re-sent within seconds; stale post-failover streams within the churn
/// window), small enough to stay flat in memory over a long game.
#[derive(Default)]
pub struct RecentUids {
    set: std::collections::HashSet<u64>,
    order: std::collections::VecDeque<u64>,
}

impl RecentUids {
    const CAP: usize = 4096;

    /// Returns `true` if the uid was newly inserted (first application),
    /// `false` if it was already known (duplicate delivery).
    pub fn insert(&mut self, uid: u64) -> bool {
        if !self.set.insert(uid) {
            return false;
        }
        self.order.push_back(uid);
        while self.order.len() > Self::CAP {
            let evicted = self.order.pop_front().expect("non-empty");
            self.set.remove(&evicted);
        }
        true
    }

    pub fn contains(&self, uid: u64) -> bool {
        self.set.contains(&uid)
    }
}

#[derive(Resource, Default)]
pub struct NetState {
    pub peers: Vec<PeerId>,
    pub my_id: Option<PeerId>,
    pub is_host: bool,
    pub snapshot_pending: Vec<PeerId>,
    pub needs_snapshot: bool,
    pub snapshot_retry_timer: f64,
    /// Set to true after the first `GameHistory` is applied.
    /// Prevents a second history replay if duplicate snapshots arrive.
    pub snapshot_applied: bool,
    /// Host-only: the next canonical sequence number to assign to a game
    /// event. Incremented every time the host sequences an event (whether
    /// locally originated or relayed from a guest). Meaningless on guests.
    pub next_seq: u32,
    /// The highest sequence number whose event has been applied locally. Used to
    /// drop a duplicate `Sequenced` delivery (same or lower `seq`) so an event is
    /// never applied twice -- the reliable channel is ordered and `seq` is
    /// monotonic, so any `seq <= last_applied_seq` has already been applied.
    /// `None` until the first event is applied.
    pub last_applied_seq: Option<u32>,
    /// All peers (including `my_id`) in canonical sorted order. Maintained by
    /// `refresh_sorted`; used by `sender_idx` for O(log n) lookup and by the
    /// host-election + turn-index logic. Empty until at least one peer is known.
    sorted_all: Vec<PeerId>,
    /// Remaining seconds of the post-reconnect resync gate (see
    /// [`RESYNC_BOOTSTRAP_SECS`]): while positive, this peer must not
    /// sequence. Set by `handle_reconnect`, decremented per frame in
    /// `handle_socket`, cleared when a `GameHistory` is installed.
    pub resync_gate_secs: f32,
    /// Seconds accumulated (via `Time` in `handle_socket`) since the peer set
    /// or our own id last changed. Host sequencing is only allowed once this
    /// exceeds [`SEQ_STABILIZE_SECS`]: two peers that join near-simultaneously
    /// each briefly elect *themselves* host, and events sequenced during that
    /// window collide in seq space and are then dropped by the other side's
    /// apply-once dedup -- a permanent, silent divergence.
    pub election_stable_secs: f32,
    /// True once this peer has ever seen at least one other peer (or runs in
    /// offline self-host mode). A peer that has *never* seen the roster must
    /// not sequence: a lone peer cannot know whether a session already exists
    /// elsewhere in the room, and its self-sequenced stream would collide with
    /// the session's canonical numbering. This is the network-side analogue of
    /// the lobby discipline (StartGame requires both factions picked, so a
    /// game cannot start solo in a networked room anyway).
    pub has_ever_peered: bool,
    /// Set when a received `Sequenced` delivery proves the local record
    /// divergent (seq conflict) or incomplete (seq gap): the next
    /// `Control::GameHistory` must be installed even if it is not "ahead" by
    /// seq alone, because the local record is known to be wrong.
    pub force_install_history: bool,
    /// Recently applied submission uids, for identity-level dedup: the same
    /// event sequenced twice under different seq numbers (transient dual-host
    /// streams) must still be applied exactly once.
    pub recent_uids: RecentUids,
}

impl NetState {
    /// Rebuild `sorted_all` from the current `peers` + `my_id`. Call this after
    /// any mutation of `peers` or after `my_id` is first set.
    pub fn refresh_sorted(&mut self) {
        self.sorted_all.clear();
        self.sorted_all.extend(self.peers.iter().copied());
        if let Some(me) = self.my_id {
            self.sorted_all.push(me);
        }
        self.sorted_all.sort();
    }

    /// Canonical sorted list of all peers including the local player.
    pub fn sorted_all(&self) -> &[PeerId] {
        &self.sorted_all
    }

    /// Return the sender index of `peer` in the canonical sorted peer list, or
    /// `None` if the ID isn't in the list (e.g. a message arriving from a peer
    /// that has just disconnected, or -- after a reconnect -- under a PeerId we
    /// no longer track). Callers record this into the permanent log, so an
    /// unknown peer must *not* be silently attributed to index 0: that would
    /// mis-credit its events to whichever peer sorts first.
    pub fn sender_idx(&self, peer: PeerId) -> Option<u8> {
        self.sorted_all.binary_search(&peer).ok().map(|i| i as u8)
    }

    /// The canonical host: the lowest-sorted peer id across all peers
    /// (including the local player). `None` until at least one peer is known.
    /// Host election re-derives from this on every peer change, so a guest is
    /// promoted automatically when the previous host disconnects (§host-relay).
    pub fn host_id(&self) -> Option<PeerId> {
        self.sorted_all.first().copied()
    }
}

#[derive(Resource)]
pub struct RoomId(pub(crate) String);

impl RoomId {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Build a `MatchboxSocket` for the given room. Used both at startup and when
/// reconnecting to a different room -- keeps ICE config and channel layout in
/// one place.
pub fn build_socket(room: &str) -> MatchboxSocket {
    let url = format!("{SIGNALING_SERVER}/{room}?next=20");
    info!(%room, %url, "opening matchbox socket");

    let ice_config = RtcIceServerConfig {
        urls: vec![
            "stun:stun.l.google.com:19302".to_string(),
            "stun:stun1.l.google.com:19302".to_string(),
        ],
        username: None,
        credential: None,
    };

    let builder = WebRtcSocketBuilder::new(&url)
        .ice_server(ice_config)
        .reconnect_attempts(None) // unlimited reconnection attempts
        .add_reliable_channel() // channel 0: game events, snapshots, identity
        .add_unreliable_channel(); // channel 1: cursors, transient UI selections

    MatchboxSocket::from(builder)
}

pub fn open_socket(mut commands: Commands, room: Res<RoomId>) {
    commands.insert_resource(build_socket(&room.0));
}

/// Reliable, ordered channel: game-mutating events, snapshots, `PlayerInfo`.
pub const CH_RELIABLE: usize = 0;
/// Unreliable channel: ephemeral display state where the latest value supersedes
/// any in-flight earlier one (cursors, viewer/browser selections).
pub const CH_UNRELIABLE: usize = 1;

/// Broadcast an ephemeral message to every peer on the unreliable channel.
/// Send failures are silently dropped -- the next sample will supersede.
pub fn broadcast_unreliable(socket: &mut MatchboxSocket, peers: &[PeerId], msg: &NetMsg) {
    if peers.is_empty() {
        return;
    }
    let Some(encoded) = enc_msg(msg) else {
        return;
    };
    let channel = socket.channel_mut(CH_UNRELIABLE);
    for &peer in peers {
        let _ = channel.try_send(encoded.clone(), peer);
    }
}

pub fn new_seed() -> u64 {
    rand::random()
}

/// Adjectives for petname room IDs. Curated short, evocative, family-friendly.
const PET_ADJECTIVES: &str = "\
ancient amber azure bold brave bright brisk bronze calm clever copper coral \
crimson crystal daring dawn dusty eager ember fierce frosty gentle gilded \
golden grand happy hidden ivory jade jolly keen lively lucky merry misty \
noble nimble onyx pearl proud quiet quick radiant rapid rosy royal ruby \
rustic shy silent silver sleepy smoky solemn sparkling stormy sunny swift \
tame tawny tender tiny twilight valiant velvet violet vivid wandering wild \
windy winter wise woven young zealous";

/// Nouns for petname room IDs. Concrete, short, no ambiguity over spelling.
const PET_NOUNS: &str = "\
albatross badger bear bison boar buffalo camel caribou cheetah cobra condor \
cougar coyote crane crow deer dingo dolphin dove eagle elk falcon ferret \
finch flamingo fox gazelle gecko goose hare hawk hedgehog heron horse hyena \
ibex jackal jaguar kestrel koala lemur leopard lion lizard llama lynx magpie \
marten meerkat mongoose moose narwhal newt ocelot orca osprey otter owl \
panda panther partridge peacock pelican penguin pony puffin puma quail \
rabbit raccoon raven reindeer salmon seal serval shark sloth sparrow stoat \
stork swan tapir tiger toucan turtle vulture walrus warbler weasel wolf \
wolverine wombat woodpecker yak zebra";

fn two_word_petname(separator: &str) -> Option<String> {
    petname::Petnames::new(PET_ADJECTIVES, "", PET_NOUNS)
        .namer(2, separator)
        .iter(&mut rand::rng())
        .next()
}

/// Generate a short hyphenated room ID like `swift-otter`.
#[cfg(target_arch = "wasm32")]
fn new_room_petname() -> String {
    two_word_petname("-").unwrap_or_else(|| format!("{:08x}", new_seed() as u32))
}

/// Generate a friendly two-word player name like `Brave Otter`, capitalised.
pub fn new_player_petname() -> String {
    let raw = two_word_petname(" ").unwrap_or_else(|| "Player".to_string());
    raw.split(' ')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn room_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::wasm_bindgen::JsValue;
        let win = web_sys::window().expect("window always available");
        let href = win.location().href().ok().unwrap_or_default();

        if let Ok(url) = web_sys::Url::new(&href) {
            if let Some(id) = url.search_params().get("room") {
                if !id.is_empty() {
                    return id;
                }
            }
        }

        let new_id = new_room_petname();

        if let Ok(url) = web_sys::Url::new(&href) {
            url.search_params().set("room", &new_id);
            if let Ok(history) = win.history() {
                let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&url.href()));
            }
        }

        new_id
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let room = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dev-room".to_string());
        info!(%room, "using room");
        room
    }
}

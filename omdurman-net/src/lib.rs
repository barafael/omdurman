use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use chrono::{DateTime, Utc};
use matchbox_socket::RtcIceServerConfig;
use omdurman_rules::MovementPoints;
use omdurman_rules::OptionalRule;
use omdurman_rules::effects::GameEffect;
use omdurman_types::{CommandScope, HexCoord, Player, Scenario, SpriteRef};
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

/// A player's stable identity across reconnects. The matchbox `PeerId`
/// changes whenever the socket is rebuilt (stall recovery, a room re-join);
/// the key does not: native builds persist it in a locked slot file in the
/// user's config directory (a relaunch reclaims it; concurrent instances get
/// distinct slots), the web keeps it per browser tab in `sessionStorage`
/// (a reload keeps it). It is announced in [`Ephemeral::PlayerInfo`]. Seats
/// are bound to keys, so a player who drops and comes back reclaims their
/// seat automatically.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PlayerKey(pub u64);

impl PlayerKey {
    /// A fresh random key.
    pub fn random() -> Self {
        Self(rand::random())
    }
}

impl std::fmt::Display for PlayerKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Who holds a [`Seat`].
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SeatHolder {
    /// A human, by their stable [`PlayerKey`].
    Human(PlayerKey),
    /// The in-game AI commander of the seat's faction; the elected host
    /// plays its turns through the ordinary sequenced-event path.
    Ai,
}

impl SeatHolder {
    /// The holder's key, if a human holds the seat.
    pub fn human(self) -> Option<PlayerKey> {
        match self {
            SeatHolder::Human(key) => Some(key),
            SeatHolder::Ai => None,
        }
    }
}

/// One seat at the table: a faction, optionally narrowed to a §1.1 command
/// scope (`None` = the whole faction), and its holder. The seat table is
/// committed by [`GameEvent::StartGame`] and only changed by recorded
/// events, so every peer and every replay agree on it.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct Seat {
    pub faction: Player,
    #[serde(default)]
    pub scope: Option<CommandScope>,
    pub holder: SeatHolder,
}

/// A game-state mutation. These are the only `NetMsg` payloads that get
/// recorded into [`GameRecord`] and replayed for late joiners. Adding a
/// variant here automatically participates in recording and replay.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, strum::IntoStaticStr)]
pub enum GameEvent {
    /// Host-committed seat table that starts the game. Each [`Seat`] binds a
    /// faction (optionally narrowed to a §1.1 command scope) to its holder --
    /// a human, identified by their stable [`PlayerKey`], or the in-game AI.
    /// Keys (not session `PeerId`s, which change on every reconnect) make the
    /// binding survive a reconnect: a returning player re-binds
    /// automatically. Recorded + replayed, so a late joiner learns the seats
    /// via the snapshot path.
    StartGame {
        /// The committed seats. Records written before seats existed carry
        /// `assignments`/`ai`/`commands` instead; those keys are ignored and
        /// the game loads seatless (reviewable, nobody bound).
        #[serde(default)]
        seats: Vec<Seat>,
        /// The scenario the host committed to. Selects which board loads
        /// (`Campaign` -> campaign map, otherwise the Fall-of-Khartoum map) and
        /// seeds the rules engine's turn track. Recorded + replayed so late
        /// joiners and history replay agree on both.
        #[serde(default)]
        scenario: Scenario,
        /// Optional rules selected by the Dervish host for a campaign game
        /// (§10.11 RiverMines, §10.21 RiverChain). Independently checkable —
        /// both may be active — and empty when none was selected or the
        /// scenario doesn't support them. (Formerly the single
        /// `optional_rule: Option<OptionalRule>`; legacy records carrying
        /// that key load with no optional rules, as if `None`.)
        #[serde(default)]
        optional_rules: Vec<OptionalRule>,
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
    /// Host-arbitrated change of one seat's holder: a newcomer claiming an
    /// abandoned seat, an abandoned seat handed to the AI, or an AI seat
    /// claimed back by a human. Applies only while seat `seat` is still held
    /// by `previous` (and a human `holder` holds no other seat), so a stale
    /// or raced decision is rejected identically on every peer. Only the
    /// elected host submits it (guests send `Control::SeatRequest`).
    SeatAssigned {
        seat: u8,
        previous: SeatHolder,
        holder: SeatHolder,
    },
    /// Host-arbitrated sub-faction takeover: `holder` (who holds no seat)
    /// gets a new seat commanding `scope` of `faction`; the carved tribes /
    /// brigades are removed from every other seat of the faction (a set
    /// emptied that way becomes `Army`).
    SeatCarved {
        faction: Player,
        scope: CommandScope,
        holder: PlayerKey,
    },
    /// The field telegram for finished game turn `turn`. Written once, by
    /// the host (its flavour model, or the turn's own events when it has
    /// none), so every peer reads the same text; recorded, so a replay or a
    /// late joiner shows it too. Presentation only: no engine effect, and
    /// the first telegram recorded for a turn wins.
    Telegram { turn: u8, text: String },
    /// The London Gazette's report paragraphs at game over, written once by
    /// the host like [`GameEvent::Telegram`].
    Gazette { paragraphs: Vec<String> },
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
    /// Display identity plus the stable [`PlayerKey`] that binds this peer
    /// to its seat. Sent reliably, targeted, to every peer on connect (and
    /// broadcast again when the name/colour changes).
    PlayerInfo {
        name: String,
        color: [u8; 3],
        key: PlayerKey,
    },
    EventViewerSelect(i32),
    /// Lobby faction pick (live preview). `None` = undecided. The authoritative
    /// binding is committed by the host via [`GameEvent::StartGame`].
    FactionChoice(Option<Player>),
    /// Lobby command-scope pick (live preview, §1.1 multi-player commands).
    /// `None` = whole faction (no scope). The authoritative binding is
    /// committed by the host via `GameEvent::StartGame`'s seats.
    CommandChoice(Option<CommandScope>),
    /// Setup-phase member readiness (§9.2/§9.3): one member of a faction has
    /// finished deploying *their* command. The faction's engine-level
    /// `ConfirmSetupReady` is submitted once every member is ready.
    SetupReady(bool),
    /// Lobby scenario pick (live preview, host-authoritative). The committed
    /// value travels in [`GameEvent::StartGame`].
    ScenarioChoice(Scenario),
    /// Lobby spectator toggle (live preview). A spectator joins the game to
    /// watch only: it is never given a seat in `StartGame`, so all action
    /// gates no-op for it. Kept
    /// separate from `FactionChoice` so peers can distinguish "spectating" from
    /// "undecided" in the lobby roster.
    SpectatorChoice(bool),
}

/// Snapshot-handshake messages. Always reliable.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Control {
    RequestSnapshot,
    /// Former install acknowledgement. Nothing consumed it (the host never
    /// read its pending-ack list), so it is no longer sent and is ignored on
    /// receipt. The variant stays so the postcard variant index of
    /// [`Control::GameHistory`] -- and with it the wire format shared with
    /// already-deployed clients -- does not shift.
    SnapshotReceived,
    GameHistory(GameRecord),
    // -- Seat claims and votes (appended: earlier variant indices are part
    //    of the wire format). Guests never submit seat events; they ask the
    //    host, which arbitrates and submits the recorded `GameEvent`. --
    /// Guest/host -> host: `requester` asks for a seat change.
    SeatRequest {
        request_id: u64,
        requester: PlayerKey,
        kind: SeatRequestKind,
    },
    /// Host -> all: a vote on `request_id` is open. Only `voters` (every
    /// connected seated human except the requester) are asked; any "no" or
    /// the deadline denies it.
    SeatVoteOpen {
        request_id: u64,
        requester: PlayerKey,
        summary: String,
        voters: Vec<PlayerKey>,
        secs_left: f32,
    },
    /// Voter -> host: a ballot on `request_id`.
    SeatVote {
        request_id: u64,
        voter: PlayerKey,
        approve: bool,
    },
    /// Host -> all: the request is decided (approved requests are followed
    /// by their recorded seat event).
    SeatVoteClosed {
        request_id: u64,
        approved: bool,
        reason: String,
    },
}

/// What a [`Control::SeatRequest`] asks for.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SeatRequestKind {
    /// Take over an abandoned seat (holder gone for the abandonment
    /// timeout). No vote.
    ClaimAbandoned { seat: u8 },
    /// Take over part of a faction (a sub-faction of tribes / brigades).
    /// Needs a unanimous vote.
    TakeOver {
        faction: Player,
        scope: CommandScope,
    },
    /// Hand an abandoned seat to the AI. Needs a unanimous vote.
    HandToAi { seat: u8 },
    /// Take an AI seat back for a human. Needs a unanimous vote.
    ClaimFromAi { seat: u8 },
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

pub use omdurman_types::net_seq::{RecentUids, SEQ_GAP_TIMEOUT_SECS};

/// A `NetMsg::Sequenced` delivery as the receive path handles it: the
/// canonical seq, the submission uid, the event, and the peer it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct SequencedDelivery {
    pub seq: u32,
    pub uid: u64,
    pub event: GameEvent,
    pub from: PeerId,
}

impl omdurman_types::net_seq::Sequenced for SequencedDelivery {
    fn seq(&self) -> u32 {
        self.seq
    }
}

/// Guest-side reorder buffer for `Sequenced` deliveries (see
/// [`omdurman_types::net_seq::ReorderBuffer`], where it is proven).
pub type ReorderBuffer = omdurman_types::net_seq::ReorderBuffer<SequencedDelivery>;

#[derive(Resource, Default)]
pub struct NetState {
    /// The connected remote peers. Private: every change goes through
    /// [`NetState::add_peer`] / [`NetState::remove_peer`], which keep
    /// `sorted_all` in step (a stale `sorted_all` mis-elects the host and
    /// once left the offline lobby without its own player).
    peers: Vec<PeerId>,
    /// The local peer's id; set through [`NetState::set_my_id`].
    my_id: Option<PeerId>,
    pub is_host: bool,
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
    /// All peers (including `my_id`) in canonical sorted order, rebuilt by
    /// every setter of `peers` / `my_id`; used by `sender_idx` for O(log n)
    /// lookup and by the host-election + turn-index logic. Empty until at
    /// least one peer is known.
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
    /// `Sequenced` deliveries that arrived past the next expected seq,
    /// waiting for the gap to fill (see [`ReorderBuffer`]).
    pub reorder: ReorderBuffer,
}

impl NetState {
    /// The connected remote peers.
    pub fn peers(&self) -> &[PeerId] {
        &self.peers
    }

    /// The local peer's id, once known.
    pub fn my_id(&self) -> Option<PeerId> {
        self.my_id
    }

    /// Set the local peer's id. Returns whether it changed.
    pub fn set_my_id(&mut self, id: Option<PeerId>) -> bool {
        let changed = self.my_id != id;
        self.my_id = id;
        self.refresh_sorted();
        changed
    }

    /// Record a connected peer. Returns whether it was new.
    pub fn add_peer(&mut self, peer: PeerId) -> bool {
        if self.peers.contains(&peer) {
            return false;
        }
        self.peers.push(peer);
        self.refresh_sorted();
        true
    }

    /// Forget a disconnected peer. Returns whether it was known.
    pub fn remove_peer(&mut self, peer: PeerId) -> bool {
        let before = self.peers.len();
        self.peers.retain(|&p| p != peer);
        self.refresh_sorted();
        self.peers.len() != before
    }

    /// Rebuild `sorted_all` from the current `peers` + `my_id`.
    fn refresh_sorted(&mut self) {
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
        let fragment = read_fragment();
        if let Some(id) = fragment_param(&fragment, "room")
            && !id.is_empty()
        {
            return id;
        }

        let new_id = new_room_petname();
        write_fragment(&with_fragment_param(&fragment, "room", Some(&new_id)));
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

#[cfg(test)]
mod receive_tests {
    use super::*;

    fn delivery(seq: u32) -> SequencedDelivery {
        SequencedDelivery {
            seq,
            uid: 1000 + u64::from(seq),
            event: GameEvent::Effect(GameEffect::AdvancePhase),
            from: PeerId(uuid::Uuid::nil()),
        }
    }

    #[test]
    fn reorder_buffer_pops_contiguous_runs_in_order() {
        let mut buf = ReorderBuffer::default();
        assert!(buf.insert(delivery(5)));
        assert!(buf.insert(delivery(3)));
        assert!(buf.insert(delivery(4)));
        // Seq 2 is missing: nothing is ready.
        assert_eq!(buf.pop_next(2), None);
        assert_eq!(buf.len(), 3);
        // Once 2 is applied elsewhere, 3, 4, 5 drain in order.
        let drained: Vec<u32> = std::iter::successors(buf.pop_next(3), |d| buf.pop_next(d.seq + 1))
            .map(|d| d.seq)
            .collect();
        assert_eq!(drained, vec![3, 4, 5]);
        assert!(buf.is_empty());
    }

    #[test]
    fn reorder_buffer_discards_entries_below_expected() {
        let mut buf = ReorderBuffer::default();
        buf.insert(delivery(4));
        buf.insert(delivery(9));
        // A history install moved the watermark to 7: seq 4 is stale.
        assert_eq!(buf.pop_next(8), None);
        assert_eq!(buf.first_seq(), Some(9));
        assert_eq!(buf.pop_next(9).map(|d| d.seq), Some(9));
    }

    #[test]
    fn reorder_buffer_times_out_once_per_stall() {
        let mut buf = ReorderBuffer::default();
        assert!(!buf.tick(10.0), "an empty buffer never times out");
        buf.insert(delivery(3));
        assert!(!buf.tick(SEQ_GAP_TIMEOUT_SECS * 0.5));
        assert!(buf.tick(SEQ_GAP_TIMEOUT_SECS));
        assert!(!buf.tick(SEQ_GAP_TIMEOUT_SECS), "reported once per stall");
        // Draining ends the stall; a new gap reports again.
        assert!(buf.pop_next(3).is_some());
        buf.insert(delivery(7));
        assert!(buf.tick(SEQ_GAP_TIMEOUT_SECS * 2.0));
    }

    #[test]
    fn recent_uids_rebuild_replaces_contents() {
        let mut uids = RecentUids::default();
        uids.insert(1);
        uids.rebuild([2, 3]);
        assert!(!uids.contains(1));
        assert!(uids.contains(2) && uids.contains(3));
    }
}

#[cfg(test)]
mod serde_tests {
    use super::*;
    use omdurman_types::{BrigadeId, CommandScope, DervishTribe};
    use std::collections::BTreeSet;

    // §1.1: a seat's command scope round-trips through serde with
    // deterministic (BTreeSet-ordered) contents, so every peer and the
    // replay see the identical binding.
    #[traceability_macro::rulebook("§1.1")]
    #[test]
    fn start_game_commands_round_trip() {
        let event = GameEvent::StartGame {
            seats: vec![
                Seat {
                    faction: Player::Dervish,
                    scope: Some(CommandScope::Tribes(BTreeSet::from([
                        DervishTribe::Jaalin,
                        DervishTribe::Baggara,
                    ]))),
                    holder: SeatHolder::Human(PlayerKey(42)),
                },
                Seat {
                    faction: Player::AngloEgyptian,
                    scope: None,
                    holder: SeatHolder::Ai,
                },
            ],
            scenario: Scenario::Campaign,
            optional_rules: Vec::new(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: GameEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, event);
        let wire: GameEvent =
            postcard::from_bytes(&postcard::to_allocvec(&event).unwrap()).unwrap();
        assert_eq!(wire, event);
        match back {
            GameEvent::StartGame { seats, .. } => {
                let Some(Seat {
                    scope: Some(CommandScope::Tribes(tribes)),
                    holder: SeatHolder::Human(PlayerKey(42)),
                    ..
                }) = seats.first()
                else {
                    panic!("expected one tribe-scoped human seat, got {seats:?}");
                };
                // BTreeSet order is canonical regardless of insertion order.
                let names: Vec<String> = tribes.iter().map(|t| t.to_string()).collect();
                assert_eq!(names, vec!["Baggara", "Jaalin"]);
            }
            other => panic!("wrong event: {other:?}"),
        }
    }

    // §1.1: records written before seats existed (`assignments` / `ai` /
    // `commands` keyed by session `PeerId`) still deserialize -- the old keys
    // are ignored and the game loads seatless, so legacy saved games stay
    // reviewable.
    #[traceability_macro::rulebook("§1.1")]
    #[test]
    fn legacy_start_game_without_commands_still_loads() {
        let legacy = serde_json::json!({
            "StartGame": {
                "assignments": [],
                "scenario": "Campaign",
                "optional_rule": null,
                "ai": ["Dervish"],
            }
        });
        let event: GameEvent = serde_json::from_value(legacy).unwrap();
        match event {
            GameEvent::StartGame { seats, .. } => assert!(seats.is_empty()),
            other => panic!("wrong event: {other:?}"),
        }
    }

    #[test]
    fn seat_events_and_controls_round_trip_on_the_wire() {
        let events = [
            GameEvent::SeatAssigned {
                seat: 2,
                previous: SeatHolder::Human(PlayerKey(5)),
                holder: SeatHolder::Ai,
            },
            GameEvent::SeatCarved {
                faction: Player::Dervish,
                scope: CommandScope::Tribes(BTreeSet::from([DervishTribe::Jaalin])),
                holder: PlayerKey(9),
            },
            GameEvent::Telegram {
                turn: 3,
                text: "Heavy fighting at the Messalamia Gate.".into(),
            },
            GameEvent::Gazette {
                paragraphs: vec!["Khartoum holds.".into(), "Gordon defiant.".into()],
            },
        ];
        for event in events {
            let msg = NetMsg::Sequenced {
                seq: 3,
                uid: 4,
                event: event.clone(),
            };
            match decode(&enc_msg(&msg).unwrap()) {
                Some(NetMsg::Sequenced { event: back, .. }) => assert_eq!(back, event),
                other => panic!("wrong message: {other:?}"),
            }
            let json = serde_json::to_string(&event).unwrap();
            assert_eq!(serde_json::from_str::<GameEvent>(&json).unwrap(), event);
        }
        let controls = [
            Control::SeatRequest {
                request_id: 1,
                requester: PlayerKey(9),
                kind: SeatRequestKind::TakeOver {
                    faction: Player::AngloEgyptian,
                    scope: CommandScope::Brigades(BTreeSet::from([BrigadeId::british(2)])),
                },
            },
            Control::SeatVoteOpen {
                request_id: 1,
                requester: PlayerKey(9),
                summary: "take over Brigades 2B".into(),
                voters: vec![PlayerKey(1), PlayerKey(2)],
                secs_left: 60.0,
            },
            Control::SeatVote {
                request_id: 1,
                voter: PlayerKey(1),
                approve: true,
            },
            Control::SeatVoteClosed {
                request_id: 1,
                approved: false,
                reason: "denied".into(),
            },
        ];
        for control in controls {
            let bytes = enc_msg(&NetMsg::Control(control.clone())).unwrap();
            let Some(NetMsg::Control(back)) = decode(&bytes) else {
                panic!("control did not decode");
            };
            assert_eq!(format!("{back:?}"), format!("{control:?}"));
        }
        // Appending keeps the existing Control indices: GameHistory is
        // still variant 2 on the wire.
        let history = NetMsg::Control(Control::GameHistory(GameRecord {
            initial_state: InitialGameState { seed: 0 },
            events: Vec::new(),
        }));
        assert_eq!(enc_msg(&history).unwrap()[..2], [3, 2]);
    }

    #[test]
    fn player_info_carries_the_player_key() {
        let msg = NetMsg::Ephemeral(Ephemeral::PlayerInfo {
            name: "Brave Otter".into(),
            color: [1, 2, 3],
            key: PlayerKey(u64::MAX),
        });
        let bytes = enc_msg(&msg).unwrap();
        match decode(&bytes) {
            Some(NetMsg::Ephemeral(Ephemeral::PlayerInfo { key, .. })) => {
                assert_eq!(key, PlayerKey(u64::MAX));
            }
            other => panic!("wrong message: {other:?}"),
        }
    }

    // §1.1: brigade scopes serialize as their printed designation set.
    #[traceability_macro::rulebook("§1.1")]
    #[test]
    fn brigade_scope_display_and_membership() {
        let mut set = BTreeSet::new();
        set.insert(BrigadeId::british(1));
        set.insert(BrigadeId::egyptian(2));
        let scope = CommandScope::Brigades(set);
        assert_eq!(scope.to_string(), "Brigades 1B, 2E");
        assert!(scope.claims_brigade(BrigadeId::egyptian(2)));
        assert!(!scope.claims_brigade(BrigadeId::british(3)));
    }
}

// ── URL fragment helpers ─────────────────────────────────────────────────────
//
// The room travels in the URL *fragment* (`#room=name`), not a query
// parameter: the browser never sends the fragment to the server, so a room
// name cannot land in access logs or referrers, changing it does not reload
// the page, and caches see one URL for the app. The fragment is a `&`-separated
// list of `key=value` pairs so other state can share it.

/// The value of `key` in a fragment, given with or without its leading `#`.
#[cfg(any(target_arch = "wasm32", test))]
pub fn fragment_param(fragment: &str, key: &str) -> Option<String> {
    let fragment = fragment.strip_prefix('#').unwrap_or(fragment);
    fragment
        .split(['&', ';'])
        .find_map(|pair| pair.strip_prefix(key)?.strip_prefix('='))
        .map(str::to_string)
}

/// `fragment` with `key` set to `value` (or removed when `None`), every other
/// pair kept in place. Returned without the leading `#`.
#[cfg(any(target_arch = "wasm32", test))]
pub fn with_fragment_param(fragment: &str, key: &str, value: Option<&str>) -> String {
    let fragment = fragment.strip_prefix('#').unwrap_or(fragment);
    let mut pairs: Vec<String> = fragment
        .split(['&', ';'])
        .filter(|pair| !pair.is_empty() && pair.split('=').next() != Some(key))
        .map(str::to_string)
        .collect();
    if let Some(value) = value {
        pairs.push(format!("{key}={value}"));
    }
    pairs.join("&")
}

/// The page's current fragment (with its leading `#`, or empty).
#[cfg(target_arch = "wasm32")]
pub fn read_fragment() -> String {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .unwrap_or_default()
}

/// Replace the page's fragment in place: no navigation, no history entry,
/// so Back does not lead to a bare page that redirects forward again.
#[cfg(target_arch = "wasm32")]
pub fn write_fragment(fragment: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let url = format!("#{fragment}");
    match window.history() {
        Ok(history) => {
            let _ = history.replace_state_with_url(
                &web_sys::wasm_bindgen::JsValue::NULL,
                "",
                Some(&url),
            );
        }
        Err(_) => {
            let _ = window.location().set_hash(fragment);
        }
    }
}

#[cfg(test)]
mod fragment_tests {
    use super::*;

    #[test]
    fn parses_a_share_link_fragment_with_leading_hash() {
        assert_eq!(fragment_param("#room=abc", "room").as_deref(), Some("abc"));
    }

    #[test]
    fn tolerates_a_fragment_without_the_hash() {
        assert_eq!(fragment_param("room=abc", "room").as_deref(), Some("abc"));
    }

    #[test]
    fn finds_the_key_among_extra_params() {
        assert_eq!(
            fragment_param("#x=1&room=q-7&y=2", "room").as_deref(),
            Some("q-7")
        );
        assert_eq!(fragment_param("#rooms=1", "room"), None);
        assert_eq!(fragment_param("", "room"), None);
    }

    #[test]
    fn values_keep_their_own_equals_signs() {
        assert_eq!(
            fragment_param("#layout=YWJj==&room=r", "layout").as_deref(),
            Some("YWJj==")
        );
    }

    #[test]
    fn setting_a_param_keeps_the_others() {
        assert_eq!(
            with_fragment_param("#layout=abc", "room", Some("r1")),
            "layout=abc&room=r1"
        );
        assert_eq!(
            with_fragment_param("#room=old&layout=abc", "room", Some("new")),
            "layout=abc&room=new"
        );
        assert_eq!(
            with_fragment_param("#room=r&layout=abc", "layout", None),
            "room=r"
        );
        assert_eq!(with_fragment_param("", "room", Some("r")), "room=r");
        assert_eq!(with_fragment_param("#room=r", "room", None), "");
    }
}

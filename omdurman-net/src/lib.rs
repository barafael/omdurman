use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use chrono::{DateTime, Utc};
use matchbox_socket::RtcIceServerConfig;
use omdurman_rules::effects::GameEffect;
use omdurman_types::{
    AnnotationsFile, HexCoord, HexsideKind, HexsideRef, MapKind, NileFlow, OverlayParams,
    SpriteAnnotation, SpriteRef, UnitGrid,
};
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

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
#[derive(Serialize, Deserialize, Clone, Debug, IntoStaticStr)]
pub enum GameEvent {
    LoadAnnotations(Box<AnnotationsFile>),
    /// Host-committed faction assignment that starts the game. Maps each
    /// player's `PeerId` (as its string form, stable within the session) to
    /// the `Player` (faction) they will command. Recorded + replayed, so a
    /// late joiner learns the bindings via the snapshot path.
    StartGame {
        assignments: Vec<(String, omdurman_rules::Player)>,
        /// The scenario the host committed to. Selects which board loads
        /// (`Campaign` -> campaign map, otherwise the Fall-of-Khartoum map) and
        /// seeds the rules engine's turn track. Recorded + replayed so late
        /// joiners and history replay agree on both.
        #[serde(default)]
        scenario: omdurman_rules::Scenario,
    },
    /// A semantic game action resolved by the rule engine (§effect system).
    Effect(GameEffect),
    MapEdit {
        /// Which board this edit applies to (§dual-map).
        #[serde(default)]
        map: MapKind,
        q: i32,
        r: i32,
        terrain: u8,
        name: String,
        /// Per-edge Nile current annotation for `is_nile` hexes (§5.11, §5.24);
        /// `None` for non-Nile hexes or hexes with no current annotated.
        #[serde(default)]
        nile_flow: Option<NileFlow>,
        /// Whether roads converge at this hex's centre or stop at the edge.
        #[serde(default)]
        is_crossroad: bool,
    },
    OverlayUpdate(MapKind, OverlayParams),
    /// Mark (or unmark) a hex inside the overlay grid as not part of the
    /// playable map -- board furniture like logos or the turn track (§dual-map).
    /// Editor-time; synced + replayed so the exclusion persists.
    ExcludeHex {
        #[serde(default)]
        map: MapKind,
        q: i32,
        r: i32,
        excluded: bool,
    },
    /// Set (or clear, when `kind` is `None`) the hexside feature on the edge
    /// between two adjacent hexes. Map-editor action; synced + replayed.
    HexsideEdit {
        /// Which board this edit applies to (§dual-map).
        #[serde(default)]
        map: MapKind,
        edge: HexsideRef,
        kind: Option<HexsideKind>,
    },
    /// Toggle a road connection between two adjacent hexes. Editor action;
    /// synced + replayed so the road graph is consistent.
    RoadEdit {
        /// Which board this edit applies to (§dual-map).
        #[serde(default)]
        map: MapKind,
        edge: HexsideRef,
        /// Whether a road should be present on this edge.
        present: bool,
    },
    /// Annotate a counter on the sprite sheet. Sprite annotations are global
    /// (the counter sheet is board-independent), so this carries no `map`.
    AnnotateSprite {
        sprite: SpriteRef,
        annotation: SpriteAnnotation,
    },
    PlaceUnit {
        sprite: SpriteRef,
        coord_q: i32,
        coord_r: i32,
        is_boat: bool,
    },
    MoveUnit {
        sprite: SpriteRef,
        to_q: i32,
        to_r: i32,
        #[serde(default)]
        cost: i16,
        /// The hexes entered, excluding the start and ending at the destination
        /// (the picker's BFS route). Lets the rules engine cost the move by
        /// terrain (§5.11), classify gunboat up/downstream steps (§5.24), and
        /// enforce the ZOC-stop rule per hex along the route (§5.26/§5.43).
        /// Empty on legacy records / direct destination-only moves, in which
        /// case the engine falls back to the supplied `cost`.
        #[serde(default)]
        path: Vec<HexCoord>,
    },
    UpdateUnitGrids(Vec<UnitGrid>),
    ShowTerrainOverlay(bool),
}

/// One entry in the canonical event log: a `GameEvent` plus the metadata
/// every peer needs to replay it deterministically.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecordedEvent {
    pub utc: DateTime<Utc>,
    pub sender_idx: u8,
    /// Canonical, host-assigned global sequence number. Identical on every
    /// peer for the same event, so all peers' logs are byte-for-byte ordered
    /// the same way (§ordering).
    pub seq: u32,
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
#[derive(Serialize, Deserialize, Clone, Debug, IntoStaticStr)]
pub enum Ephemeral {
    CursorPos {
        x: f32,
        y: f32,
    },
    PlayerInfo {
        name: String,
        color_r: u8,
        color_g: u8,
        color_b: u8,
    },
    EventViewerSelect(i32),
    /// Notify peers which sprite the sender has selected in the Units browser.
    BrowserSelect {
        sprite: SpriteRef,
    },
    /// Lobby faction pick (live preview). `None` = undecided. The authoritative
    /// binding is committed by the host via [`GameEvent::StartGame`].
    FactionChoice(Option<omdurman_rules::Player>),
    /// Lobby scenario pick (live preview, host-authoritative). The committed
    /// value travels in [`GameEvent::StartGame`].
    ScenarioChoice(omdurman_rules::Scenario),
    /// Lobby spectator toggle (live preview). A spectator joins the game to
    /// watch only: it is never placed in the authoritative faction binding
    /// (`StartGame` assignments), so all action gates no-op for it. Kept
    /// separate from `FactionChoice` so peers can distinguish "spectating" from
    /// "undecided" in the lobby roster.
    SpectatorChoice(bool),
}

/// Snapshot-handshake messages. Always reliable.
#[derive(Serialize, Deserialize, Clone, Debug, IntoStaticStr)]
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
#[derive(Serialize, Deserialize, Clone, Debug, IntoStaticStr)]
pub enum NetMsg {
    /// Unsequenced game-event submission, sent guest->host. The host orders it
    /// and rebroadcasts as [`NetMsg::Sequenced`]; it is never applied directly.
    Game(GameEvent),
    /// Canonical, host-sequenced game event, sent host->all. This is the only
    /// form that is applied to the world and appended to the event log.
    Sequenced {
        seq: u32,
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
    /// mis-credit its events to whichever peer sorts first. Callers pick an
    /// explicit fallback (`sender_idx_or_recorded`) instead.
    pub fn sender_idx(&self, peer: PeerId) -> Option<u8> {
        self.sorted_all.binary_search(&peer).ok().map(|i| i as u8)
    }

    /// Sender index for recording, with an explicit sentinel for an unknown
    /// peer. `u8::MAX` marks "sender not resolvable at record time" -- distinct
    /// from a real index -- so a reconnected/departed peer's events are flagged
    /// rather than misattributed to peer 0.
    pub const SENDER_UNKNOWN: u8 = u8::MAX;

    pub fn sender_idx_or_recorded(&self, peer: PeerId) -> u8 {
        self.sender_idx(peer).unwrap_or(Self::SENDER_UNKNOWN)
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
pub struct RoomId(pub String);

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
    commands.spawn(build_socket(&room.0));
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
#[allow(dead_code)]
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

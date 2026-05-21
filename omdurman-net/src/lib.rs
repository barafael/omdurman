use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use chrono::{DateTime, Utc};
use matchbox_socket::RtcIceServerConfig;
use omdurman_types::{AnnotationsFile, OverlayParams, SpriteAnnotation, UnitGrid};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use strum::{FromRepr, IntoStaticStr};

pub const SIGNALING_SERVER: &str = if let Some(s) = option_env!("MATCHBOX_SERVER") {
    s
} else {
    "wss://omdurman-matchbox.fly.dev"
};

// ── Event-sourced game record ─────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitialGameState {
    pub seed: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameEvent {
    pub utc: DateTime<Utc>,
    pub sender_idx: u8,
    pub seq: u32,
    pub payload: EventPayload,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventPayload {
    LoadAnnotations(AnnotationsFile),
    Action(u32),
    ModeSwitch(EditorMode),
    MapEdit {
        q: i32,
        r: i32,
        terrain: u8,
        name: String,
    },
    OverlayUpdate(OverlayParams),
    AnnotateSprite {
        section_name: String,
        col: u32,
        row: u32,
        annotation: SpriteAnnotation,
    },
    PlaceUnit {
        section_name: String,
        col: u32,
        row: u32,
        coord_q: i32,
        coord_r: i32,
        is_boat: bool,
    },
    MoveUnit {
        section_name: String,
        col: u32,
        row: u32,
        to_q: i32,
        to_r: i32,
    },
    UpdateUnitGrids(Vec<UnitGrid>),
    ShowTerrainOverlay(bool),
    PlayerInfo {
        name: String,
        color_r: u8,
        color_g: u8,
        color_b: u8,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameRecord {
    pub initial_state: InitialGameState,
    pub events: Vec<GameEvent>,
}

// ── Wire protocol ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetMsg {
    Action(u32),
    ModeSwitch(EditorMode),
    MapEdit {
        q: i32,
        r: i32,
        terrain: u8,
        name: String,
    },
    OverlayUpdate(OverlayParams),
    AnnotateSprite {
        section_name: String,
        col: u32,
        row: u32,
        annotation: SpriteAnnotation,
    },
    PlaceUnit {
        section_name: String,
        col: u32,
        row: u32,
        coord_q: i32,
        coord_r: i32,
        is_boat: bool,
    },
    MoveUnit {
        section_name: String,
        col: u32,
        row: u32,
        to_q: i32,
        to_r: i32,
    },
    UpdateUnitGrids(Vec<UnitGrid>),
    ShowTerrainOverlay(bool),
    PlayerInfo {
        name: String,
        color_r: u8,
        color_g: u8,
        color_b: u8,
    },
    CursorPos {
        x: f32,
        y: f32,
    },
    RequestSnapshot,
    SnapshotReceived,
    LoadAnnotations(AnnotationsFile),
    GameHistory(GameRecord),
    EventViewerSelect(i32),
    /// Notify peers which sprite the sender has selected in the Units browser.
    BrowserSelect {
        section_name: String,
        col: u32,
        row: u32,
    },
}

pub fn enc_msg(msg: &NetMsg) -> Box<[u8]> {
    postcard::to_allocvec(msg)
        .expect("NetMsg is always serializable")
        .into_boxed_slice()
}

pub fn decode(raw: &[u8]) -> Option<NetMsg> {
    postcard::from_bytes(raw)
        .inspect_err(|e| warn!("matchbox decode error: {e}"))
        .ok()
}

#[derive(Resource)]
pub struct NetState {
    pub peers: Vec<PeerId>,
    pub my_id: Option<PeerId>,
    pub is_host: bool,
    pub snapshot_pending: Vec<PeerId>,
    pub needs_snapshot: bool,
    pub snapshot_retry_timer: f64,
    /// Set to true after the first `GameHistory` is applied.
    /// Prevents a second history replay if both the proactive send
    /// and the RequestSnapshot response arrive in the same session.
    pub snapshot_applied: bool,
}

impl Default for NetState {
    fn default() -> Self {
        Self {
            peers: Vec::new(),
            my_id: None,
            is_host: false,
            snapshot_pending: Vec::new(),
            needs_snapshot: false,
            snapshot_retry_timer: 0.0,
            snapshot_applied: false,
        }
    }
}

impl NetState {
    /// Return the sender index of `peer` in the canonical sorted peer list
    /// (which includes the local player). Returns 0 if the ID isn't found.
    pub fn sender_idx(&self, peer: PeerId) -> u8 {
        let mut all: Vec<PeerId> = self.peers.clone();
        if let Some(me) = self.my_id {
            all.push(me);
        }
        all.sort();
        all.iter().position(|&p| p == peer).unwrap_or(0) as u8
    }
}

#[derive(
    Resource,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Default,
    Serialize,
    Deserialize,
    FromRepr,
    IntoStaticStr,
)]
#[repr(u8)]
pub enum EditorMode {
    #[default]
    Normal,
    Overlay,
    Editor,
    UnitSheet,
    Units,
    Dice,
    Secret,
    EventViewer,
}

#[derive(Resource)]
pub struct GameRng(ChaCha8Rng);

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }

    pub fn random_u32(&mut self) -> u32 {
        use rand::RngExt;
        self.0.random::<u32>()
    }
}

#[derive(Resource)]
pub struct RoomId(pub String);

pub fn open_socket(mut commands: Commands, room: Res<RoomId>) {
    let url = format!("{}/{}?next=20", SIGNALING_SERVER, room.0);
    info!(room = %room.0, %url, "opening matchbox socket");

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
        .add_reliable_channel();

    commands.spawn(MatchboxSocket::from(builder));
}

pub fn new_seed() -> u64 {
    rand::random()
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

        let new_id = format!("{:08x}", new_seed() as u32);

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
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dev-room".to_string())
    }
}

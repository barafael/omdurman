use std::collections::HashMap;

use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use omdurman_types::{
    HexCoord, HexData, OverlayParams, SpriteAnnotation, SpriteAnnotations, UnitGrid,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub const SIGNALING_SERVER: &str = if let Some(s) = option_env!("MATCHBOX_SERVER") {
    s
} else {
    "wss://omdurman-matchbox.fly.dev"
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlacedUnitSnapshot {
    pub section_name: String,
    pub col: u32,
    pub row: u32,
    pub coord_q: i32,
    pub coord_r: i32,
    pub movement: u32,
    pub is_boat: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameStateSnapshot {
    pub hexes: HashMap<HexCoord, HexData>,
    pub overlay: OverlayParams,
    pub editor_mode: u8,
    pub annotations: SpriteAnnotations,
    pub unit_grids: Vec<UnitGrid>,
    pub show_terrain_overlay: bool,
    pub placed_units: Vec<PlacedUnitSnapshot>,
    pub seed: u64,
    pub current_turn: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetMsg {
    Seed(u64),
    Action(u32),
    ModeSwitch(u8),
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
    SyncState {
        seed: u64,
        current_turn: usize,
    },
    ShowTerrainOverlay(bool),
    FullStateSnapshot(GameStateSnapshot),
    RequestSnapshot,
    SnapshotReceived,
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

#[derive(Resource, Default)]
pub struct NetState {
    pub peers: Vec<PeerId>,
    pub is_host: bool,
    pub current_seed: Option<u64>,
    pub snapshot_pending: Vec<PeerId>,
    pub needs_snapshot: bool,
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
    commands.spawn(MatchboxSocket::new_reliable(url));
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

use bevy::prelude::*;
use bevy_matchbox::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

pub const SIGNALING_SERVER: &str = if let Some(s) = option_env!("MATCHBOX_SERVER") {
    s
} else {
    "wss://omdurman-matchbox.fly.dev"
};

#[derive(Serialize, Deserialize, Debug)]
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
    pub peer: Option<PeerId>,
    pub is_host: bool,
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
    let url = format!("{}/{}?next=2", SIGNALING_SERVER, room.0);
    info!(room = %room.0, %url, "opening matchbox socket");
    commands.spawn(MatchboxSocket::new_reliable(url));
}

pub fn new_seed() -> u64 {
    rand::random()
}

pub fn room_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let hash = web_sys::window()
            .and_then(|w| w.location().hash().ok())
            .unwrap_or_default();
        let id = hash.trim_start_matches('#').to_string();
        if id.is_empty() {
            let new_id = format!("{:08x}", new_seed() as u32);
            if let Some(win) = web_sys::window() {
                let _ = win.location().set_hash(&new_id);
            }
            new_id
        } else {
            id
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dev-room".to_string())
    }
}

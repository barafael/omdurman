use bevy::prelude::*;
use omdurman_net::{
    EventPayload, GameEvent, GameRecord, GameRng, InitialGameState, NetMsg, new_seed,
};
use omdurman_types::AnnotationsFile;

#[derive(Resource, Default)]
pub struct GameRecorder {
    pub record: Option<GameRecord>,
    host_seq: u32,
    pub annotations_sent: bool,
    dirty: bool,
    #[cfg(not(target_arch = "wasm32"))]
    file_path: String,
}

impl GameRecorder {
    pub fn init(seed: u64) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let file_path = {
            let ts = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ");
            format!("games/game_{ts}.ron")
        };
        Self {
            record: Some(GameRecord {
                initial_state: InitialGameState { seed },
                events: Vec::new(),
            }),
            host_seq: 0,
            annotations_sent: false,
            dirty: false,
            #[cfg(not(target_arch = "wasm32"))]
            file_path,
        }
    }

    /// Append a game-mutating `NetMsg` to `GameRecorder`.
    /// `sender_idx` identifies which peer sent the event (0 = host / local player).
    /// Converts relevant NetMsg variants into `EventPayload` + wraps in `GameEvent`.
    /// Returns `true` if an event was recorded.
    pub fn push_event(&mut self, msg: &NetMsg, sender_idx: u8) -> bool {
        let record = match &mut self.record {
            Some(r) => r,
            None => return false,
        };
        let utc = chrono::Utc::now();
        let seq = self.host_seq;
        self.host_seq += 1;
        let payload = match msg {
            NetMsg::LoadAnnotations(f) => Some(EventPayload::LoadAnnotations(f.clone())),
            NetMsg::Action(d) => Some(EventPayload::Action(*d)),
            NetMsg::ModeSwitch(m) => Some(EventPayload::ModeSwitch(*m)),
            NetMsg::MapEdit {
                q,
                r,
                terrain,
                name,
            } => Some(EventPayload::MapEdit {
                q: *q,
                r: *r,
                terrain: *terrain,
                name: name.clone(),
            }),
            NetMsg::OverlayUpdate(p) => Some(EventPayload::OverlayUpdate(p.clone())),
            NetMsg::AnnotateSprite {
                section_name,
                col,
                row,
                annotation,
            } => Some(EventPayload::AnnotateSprite {
                section_name: section_name.clone(),
                col: *col,
                row: *row,
                annotation: annotation.clone(),
            }),
            NetMsg::PlaceUnit {
                section_name,
                col,
                row,
                coord_q,
                coord_r,
                is_boat,
            } => Some(EventPayload::PlaceUnit {
                section_name: section_name.clone(),
                col: *col,
                row: *row,
                coord_q: *coord_q,
                coord_r: *coord_r,
                is_boat: *is_boat,
            }),
            NetMsg::MoveUnit {
                section_name,
                col,
                row,
                to_q,
                to_r,
            } => Some(EventPayload::MoveUnit {
                section_name: section_name.clone(),
                col: *col,
                row: *row,
                to_q: *to_q,
                to_r: *to_r,
            }),
            NetMsg::UpdateUnitGrids(g) => Some(EventPayload::UpdateUnitGrids(g.clone())),
            NetMsg::ShowTerrainOverlay(v) => Some(EventPayload::ShowTerrainOverlay(*v)),
            // PlayerInfo is identity metadata, not game state.  It is re-sent
            // live by send_player_info_on_connect and must not be replayed
            // (no stable PeerId↔sender mapping across sessions).
            _ => None, // non-recorded: CursorPos, EventViewerSelect, PlayerInfo, …
        };
        if let Some(payload) = payload {
            record.events.push(GameEvent {
                utc,
                sender_idx,
                seq,
                payload,
            });
            self.dirty = true;
            true
        } else {
            false
        }
    }
}

/// Initialize the game record on the very first frame.
/// Every peer creates a local record;  the canonical host's record is the
/// one distributed to late joiners via GameHistory.
pub fn init_game_record(mut commands: Commands, mut recorder: ResMut<GameRecorder>) {
    if recorder.record.is_some() {
        return;
    }
    let seed = new_seed();
    commands.insert_resource(GameRng::from_seed(seed));
    *recorder = GameRecorder::init(seed);
    info!(seed, "game record initialised");
}

/// Host-only system: record all game-mutating messages in `PendingEdits.items`
/// (host-initiated actions that will be broadcast next frame).
pub fn record_host_events(
    mut recorder: ResMut<GameRecorder>,
    pending: Res<super::PendingEdits>,
    net: Res<omdurman_net::NetState>,
    turn: Res<super::TurnState>,
) {
    // Treat this peer as the canonical recorder if either:
    //  - we are the negotiated host, or
    //  - there are no peers yet (solo host before any guests join).
    if !net.is_host && !net.peers.is_empty() {
        return;
    }
    let my_idx = turn.my_index as u8;
    for msg in &pending.items {
        recorder.push_event(msg, my_idx);
    }
}

/// Host-only system: emits LoadAnnotations as the first event.
/// Runs once after the game record is initialised.
pub fn host_emit_annotations(
    mut recorder: ResMut<GameRecorder>,
    mut pending: ResMut<super::PendingEdits>,
) {
    if recorder.annotations_sent {
        return;
    }
    if recorder.record.is_none() {
        return;
    }
    recorder.annotations_sent = true;

    let ron_str = include_str!("../assets/annotations.ron");
    let file: AnnotationsFile = match ron::from_str(ron_str) {
        Ok(f) => f,
        Err(e) => {
            warn!("failed to parse annotations.ron for host emission: {e}");
            return;
        }
    };

    info!("host: emitting LoadAnnotations as first event");
    let msg = NetMsg::LoadAnnotations(file);
    pending.items.push(msg);
}

/// Write the game record to disk (native only).
/// On WASM, keep in memory; user can download via a button.
pub fn flush_game_record(mut recorder: ResMut<GameRecorder>) {
    if !recorder.dirty {
        return;
    }
    recorder.dirty = false;
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = recorder.file_path.clone();
        if let Some(ref record) = recorder.record {
            use ron::ser::PrettyConfig;

            if let Some(parent) = std::path::Path::new(&path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(ron_str) = ron::ser::to_string_pretty(record, PrettyConfig::default()) {
                let _ = std::fs::write(&path, &ron_str);
                info!(path = %path, "game record flushed");
            }
        }
    }
}

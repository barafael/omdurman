use bevy::prelude::*;
use omdurman_net::{
    GameEvent, GameRecord, GameRng, InitialGameState, NetMsg, RecordedEvent, new_seed,
};
use omdurman_types::AnnotationsFile;

/// Append-only log of every `GameEvent` this peer has seen.
///
/// Every peer keeps its own copy. They agree because they all observe the
/// same reliable-channel event stream in the same order. The host's record
/// is the one distributed to late joiners via `Control::GameHistory`, but
/// any peer's record would do.
///
/// On native the log is an append-only JSONL file (`games/game_{ts}.jsonl`):
/// the first line is `{"seed":<n>}`, each subsequent line is a
/// `RecordedEvent` in JSON.
#[derive(Resource, Default)]
pub struct GameRecorder {
    pub record: Option<GameRecord>,
    host_seq: u32,
    pub annotations_sent: bool,
    dirty: bool,
    /// How many events have been flushed to disk.
    flushed_count: usize,
    #[cfg(not(target_arch = "wasm32"))]
    events_path: String,
}

impl GameRecorder {
    pub fn init(seed: u64) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let events_path = {
            let _ = std::fs::create_dir_all("games");
            let ts = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ");
            let path = format!("games/game_{ts}.jsonl");
            // Write seed line
            if let Ok(mut f) = std::fs::File::create(&path) {
                use std::io::Write;
                let _ = write!(f, r#"{{"seed":{seed}}}"#);
                let _ = writeln!(f);
            }
            path
        };
        Self {
            record: Some(GameRecord {
                initial_state: InitialGameState { seed },
                events: Vec::new(),
            }),
            host_seq: 0,
            annotations_sent: false,
            dirty: false,
            flushed_count: 0,
            #[cfg(not(target_arch = "wasm32"))]
            events_path,
        }
    }

    /// Replace the in-memory record with a received `GameHistory` snapshot
    /// (late-joiner path). Resets the flush cursor so the next
    /// [`flush_game_record`] writes the received events to disk.
    pub(crate) fn install_history(&mut self, record: GameRecord) {
        self.record = Some(record);
        self.flushed_count = 0;
        self.dirty = true;
    }

    /// Append `event` to the record, tagged with `sender_idx`. Returns
    /// `true` if the event was actually recorded (false if the recorder
    /// hasn't been initialised yet).
    pub fn push_event(&mut self, event: &GameEvent, sender_idx: u8) -> bool {
        let Some(record) = &mut self.record else {
            return false;
        };
        let utc = chrono::Utc::now();
        let seq = self.host_seq;
        self.host_seq += 1;
        record.events.push(RecordedEvent {
            utc,
            sender_idx,
            seq,
            payload: event.clone(),
        });
        self.dirty = true;
        true
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

/// Record every game-mutating message this peer is *about to broadcast*.
pub fn record_outgoing_broadcasts(
    mut recorder: ResMut<GameRecorder>,
    pending: Res<super::PendingEdits>,
    turn: Res<super::TurnState>,
) {
    let my_idx = turn.my_index as u8;
    for msg in &pending.outgoing_broadcast {
        if let NetMsg::Game(ev) = msg {
            recorder.push_event(ev, my_idx);
        }
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
    pending
        .outgoing_broadcast
        .push(NetMsg::Game(GameEvent::LoadAnnotations(file)));
}

/// Append unreleased events to the JSONL file (native only).
/// On WASM, keep everything in memory; user can download via a button.
pub fn flush_game_record(mut recorder: ResMut<GameRecorder>) {
    if !recorder.dirty {
        return;
    }
    recorder.dirty = false;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(ref record) = recorder.record else {
            return;
        };
        let new_events = &record.events[recorder.flushed_count..];
        if new_events.is_empty() {
            return;
        }
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open(&recorder.events_path)
        {
            for ev in new_events {
                if let Ok(line) = serde_json::to_string(ev) {
                    let _ = writeln!(f, "{line}");
                }
            }
            recorder.flushed_count = record.events.len();
        }
    }
}

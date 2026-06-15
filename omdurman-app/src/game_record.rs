use crate::GameRng;
use bevy::prelude::*;
use omdurman_net::{GameEvent, GameRecord, InitialGameState, NetMsg, RecordedEvent, new_seed};
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
            if let Err(error) = std::fs::create_dir_all("games") {
                warn!(%error, "failed to create games directory");
            }
            let ts = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%SZ");
            let path = format!("games/game_{ts}.jsonl");
            // Write the seed header line.
            match std::fs::File::create(&path) {
                Ok(mut f) => {
                    use std::io::Write;
                    if let Err(error) = writeln!(f, r#"{{"seed":{seed}}}"#) {
                        warn!(%error, %path, "failed to write seed header");
                    }
                }
                Err(error) => warn!(%error, %path, "failed to create game record file"),
            }
            path
        };
        Self {
            record: Some(GameRecord {
                initial_state: InitialGameState { seed },
                events: Vec::new(),
            }),
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

    /// Append `event` to the record, tagged with `sender_idx` and the
    /// canonical host-assigned `seq` (§ordering). Idempotent: a `seq` already
    /// present is ignored, guarding against duplicate delivery of a sequenced
    /// event. Returns `true` if the event was actually recorded (false if the
    /// recorder hasn't been initialised yet, or the `seq` was a duplicate).
    pub fn push_event(&mut self, event: &GameEvent, sender_idx: u8, seq: u32) -> bool {
        let Some(record) = &mut self.record else {
            return false;
        };
        if record.events.iter().any(|e| e.seq == seq) {
            return false;
        }
        record.events.push(RecordedEvent {
            utc: chrono::Utc::now(),
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

/// Host-only system: emits LoadAnnotations as the first game event.
///
/// Only the sequencer (the elected host, or a solo player with no peers yet)
/// may originate this -- under host-relay a guest emitting it would submit a
/// duplicate `LoadAnnotations` to the host. Runs once after the game record is
/// initialised.
pub fn host_emit_annotations(
    mut recorder: ResMut<GameRecorder>,
    mut pending: ResMut<super::PendingEdits>,
    net: Res<omdurman_net::NetState>,
) {
    if recorder.annotations_sent {
        return;
    }
    if recorder.record.is_none() {
        return;
    }
    // Only the sequencer originates game events.
    if !(net.is_host || net.peers.is_empty()) {
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
        .push(NetMsg::Game(GameEvent::LoadAnnotations(Box::new(file))));
}

/// Append unreleased events to the JSONL file (native only).
/// On WASM, keep everything in memory; user can download via a button.
pub fn flush_game_record(mut recorder: ResMut<GameRecorder>) {
    if !recorder.dirty {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        // On WASM everything stays in memory; the user downloads it via a
        // button, so there's nothing to write -- just clear the flag.
        recorder.dirty = false;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(ref record) = recorder.record else {
            recorder.dirty = false;
            return;
        };
        let new_events = &record.events[recorder.flushed_count..];
        if new_events.is_empty() {
            recorder.dirty = false;
            return;
        }
        use std::io::Write;
        // `dirty` stays set until the append succeeds, so a failed open or
        // write is retried on the next tick rather than silently dropping
        // the events.
        let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open(&recorder.events_path)
            .inspect_err(|error| {
                warn!(%error, path = %recorder.events_path, "failed to open game record for append; will retry");
            })
        else {
            return;
        };
        let mut all_written = true;
        for ev in new_events {
            let Ok(line) = serde_json::to_string(ev)
                .inspect_err(|error| warn!(%error, "failed to serialise recorded event; skipping"))
            else {
                continue;
            };
            if let Err(error) = writeln!(f, "{line}") {
                warn!(%error, path = %recorder.events_path, "failed to write recorded event; will retry");
                all_written = false;
                break;
            }
        }
        if all_written {
            recorder.flushed_count = record.events.len();
            recorder.dirty = false;
        }
    }
}

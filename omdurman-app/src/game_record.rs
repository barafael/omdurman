use crate::GameRng;
use bevy::prelude::*;
use omdurman_net::{GameEvent, GameRecord, InitialGameState, RecordedEvent, new_seed};

/// Root directory holding one directory per game (native only).
#[cfg(not(target_arch = "wasm32"))]
pub const GAMES_DIR: &str = "games";

/// Append-only log of every `GameEvent` this peer has seen.
///
/// Every peer keeps its own copy. They agree because they all observe the
/// same reliable-channel event stream in the same order. The host's record
/// is the one distributed to late joiners via `Control::GameHistory`, but
/// any peer's record would do.
///
/// On native each game gets its own directory (`games/game_{ts}_{suffix}/`)
/// holding the append-only event log (`events.jsonl`: the first line is
/// `{"seed":<n>}`, each subsequent line is a `RecordedEvent` in JSON) plus
/// the flavour-text artifacts written by the telegram / newspaper systems
/// (`telegrams.md`, `newspaper.md`).
#[derive(Resource, Default)]
pub struct GameRecorder {
    pub record: Option<GameRecord>,

    dirty: bool,
    /// How many events have been flushed to disk.
    flushed_count: usize,
    #[cfg(not(target_arch = "wasm32"))]
    events_path: String,
    /// This game's artifact directory (`games/game_{ts}_{suffix}`), created
    /// by [`GameRecorder::init`]. Empty until then.
    #[cfg(not(target_arch = "wasm32"))]
    game_dir: String,
}

impl GameRecorder {
    pub fn init(seed: u64) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let (events_path, game_dir) = {
            // Millisecond precision plus a per-process random suffix so two
            // local instances starting in the same second cannot land on the
            // same directory and interleave their appends into one corrupt
            // file (which produced doubled `}{ ` lines). Each instance records
            // to its own directory.
            let ts = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S-%3fZ");
            let suffix = format!("{:04x}", omdurman_net::new_seed() as u16);
            let dir = format!("{GAMES_DIR}/game_{ts}_{suffix}");
            if let Err(error) = std::fs::create_dir_all(&dir) {
                warn!(%error, %dir, "failed to create game directory");
            }
            let path = format!("{dir}/events.jsonl");
            // Write the seed header line (with the engine version stamp so
            // replays can detect cross-version divergence).
            match std::fs::File::create(&path) {
                Ok(mut f) => {
                    use std::io::Write;
                    let version = omdurman_rules::RULES_VERSION;
                    if let Err(error) =
                        writeln!(f, r#"{{"seed":{seed},"rules_version":{version}}}"#)
                    {
                        warn!(%error, %path, "failed to write seed header");
                    }
                }
                Err(error) => warn!(%error, %path, "failed to create game record file"),
            }
            (path, dir)
        };
        Self {
            record: Some(GameRecord {
                initial_state: InitialGameState {
                    seed,
                    rules_version: omdurman_rules::RULES_VERSION,
                },
                events: Vec::new(),
            }),

            dirty: false,
            flushed_count: 0,
            #[cfg(not(target_arch = "wasm32"))]
            events_path,
            #[cfg(not(target_arch = "wasm32"))]
            game_dir,
        }
    }

    /// This game's artifact directory (`games/game_{ts}_{suffix}`), where the
    /// telegram / newspaper flavour text is persisted. `None` on wasm (no
    /// filesystem) or before [`GameRecorder::init`].
    pub fn artifacts_dir(&self) -> Option<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            (!self.game_dir.is_empty()).then(|| self.game_dir.clone())
        }
        #[cfg(target_arch = "wasm32")]
        {
            None
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
    pub fn push_event(&mut self, event: &GameEvent, sender_idx: Option<u8>, seq: u32) -> bool {
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

// -- Load a saved game from disk (native only) ----------------------------

/// Errors from reading a game record file (e.g. `games/<game>/events.jsonl`)
/// back into a [`GameRecord`].
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, thiserror::Error)]
pub enum LoadRecordError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("empty record file {path}: missing seed header")]
    Empty { path: String },
    #[error("bad seed header in {path}: {source}")]
    SeedHeader {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("bad event on line {line} of {path}: {source}")]
    Event {
        path: String,
        line: usize,
        #[source]
        source: serde_json::Error,
    },
}

/// The seed header line written first in every record file.
#[cfg(not(target_arch = "wasm32"))]
#[derive(serde::Deserialize)]
struct SeedHeader {
    seed: u64,
    /// `0` for pre-stamping legacy headers; the writer always stamps the
    /// current [`omdurman_rules::RULES_VERSION`].
    #[serde(default)]
    rules_version: u32,
}

/// Load a game record file (written by [`flush_game_record`]) back into a
/// [`GameRecord`]: the first line is the `{"seed":<u64>}` header, each remaining
/// non-empty line is a JSON [`RecordedEvent`]. Mirrors the writer format exactly.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_record_from_jsonl(path: &str) -> Result<GameRecord, LoadRecordError> {
    let text = std::fs::read_to_string(path).map_err(|source| LoadRecordError::Io {
        path: path.to_string(),
        source,
    })?;
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().ok_or_else(|| LoadRecordError::Empty {
        path: path.to_string(),
    })?;
    let SeedHeader {
        seed,
        rules_version,
    } = serde_json::from_str(header).map_err(|source| LoadRecordError::SeedHeader {
        path: path.to_string(),
        source,
    })?;
    if rules_version != omdurman_rules::RULES_VERSION {
        warn!(
            record = path,
            record_version = rules_version,
            engine_version = omdurman_rules::RULES_VERSION,
            "game record was written by a different rules engine version; \
             replay may reject events or resolve them differently"
        );
    }
    let mut events = Vec::new();
    // Line numbers are 1-based and the header is line 1, so events start at 2.
    for (idx, line) in lines.enumerate() {
        let event: RecordedEvent =
            serde_json::from_str(line).map_err(|source| LoadRecordError::Event {
                path: path.to_string(),
                line: idx + 2,
                source,
            })?;
        events.push(event);
    }
    Ok(GameRecord {
        initial_state: InitialGameState {
            seed,
            rules_version,
        },
        events,
    })
}

/// Minimal metadata read straight off a [`GameRecord`], for the lobby's saved-
/// games list. All fields are cheap to extract from the raw event stream -- no
/// engine replay (an exact turn count would need the board loaded, the heavy
/// review path). `scenario` is `None` for records with no `StartGame` yet.
#[derive(Clone, Debug)]
pub struct GameMeta {
    pub scenario: Option<omdurman_types::Scenario>,
    /// Number of recorded events in the log.
    pub events: usize,
    /// UTC timestamp of the last recorded event (roughly when the game was last
    /// played). `None` for an empty log.
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
}

/// Extract [`GameMeta`] from a record by scanning its events -- the scenario is
/// carried by the (first) [`GameEvent::StartGame`], the rest is bookkeeping.
#[cfg(not(target_arch = "wasm32"))]
pub fn game_meta(record: &GameRecord) -> GameMeta {
    let scenario = record.events.iter().find_map(|e| match &e.payload {
        GameEvent::StartGame { scenario, .. } => Some(*scenario),
        _ => None,
    });
    GameMeta {
        scenario,
        events: record.events.len(),
        last_played: record.events.last().map(|e| e.utc),
    }
}

/// A saved game on disk plus the metadata shown for it in the lobby list.
#[derive(Clone, Debug)]
pub struct SavedGame {
    #[cfg(not(target_arch = "wasm32"))]
    pub path: String,
    pub name: String,
    /// `None` if the file could not be parsed (shown as unreadable in the UI).
    pub meta: Option<GameMeta>,
}

/// Cached list of saved games for the lobby sub-tab. Refreshed on entering the
/// lobby (and by the tab's refresh button) rather than re-read + re-parsed every
/// egui frame -- parsing every `game_*.jsonl` per frame would be wasteful. Stays
/// empty on wasm, which has no saved-game files on disk.
#[derive(Resource, Default)]
pub struct SavedGamesCache {
    pub games: Vec<SavedGame>,
    /// Set once the cache has been populated at least once, so the UI can tell
    /// "not scanned yet" from "scanned, none found".
    pub loaded: bool,
}

impl SavedGamesCache {
    /// (Re)scan [`GAMES_DIR`] and parse each file's metadata, newest first. A
    /// no-op on wasm (no on-disk saved games).
    pub fn refresh(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.games = list_saved_games()
                .into_iter()
                .map(|(path, name)| {
                    let meta = load_record_from_jsonl(&path)
                        .inspect_err(
                            |error| warn!(%error, %path, "failed to read saved-game metadata"),
                        )
                        .ok()
                        .map(|record| game_meta(&record));
                    SavedGame { path, name, meta }
                })
                .collect();
        }
        self.loaded = true;
    }
}

/// Refresh the saved-games cache whenever the lobby is entered, so the sub-tab
/// shows an up-to-date list without re-parsing files every frame.
pub fn refresh_saved_games_on_lobby(mut cache: ResMut<SavedGamesCache>) {
    cache.refresh();
}

/// List saved games in [`GAMES_DIR`], newest first, as `(path, name)`: one
/// `game_*/` directory per game, its record at `events.jsonl`. Returns an
/// empty list if the directory is missing or unreadable.
#[cfg(not(target_arch = "wasm32"))]
pub fn list_saved_games() -> Vec<(String, String)> {
    let Ok(entries) = std::fs::read_dir(GAMES_DIR) else {
        return Vec::new();
    };
    let mut games: Vec<(String, String)> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let name = path.file_name()?.to_str()?.to_string();
            if !path.is_dir() || !name.starts_with("game_") {
                return None;
            }
            let events = path.join("events.jsonl");
            events.is_file().then(|| Some((events.to_str()?.to_string(), name)))?
        })
        .collect();
    // Names embed a sortable UTC timestamp, so a reverse lexical sort puts
    // the newest game first.
    games.sort_by(|a, b| b.1.cmp(&a.1));
    games
}

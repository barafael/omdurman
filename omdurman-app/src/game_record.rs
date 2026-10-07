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
    /// The on-disk log no longer matches an append-only prefix of `record`
    /// (a history was installed, or the record was reset for a same-room
    /// resync): the next flush truncates the file and rewrites the header
    /// plus every event, instead of appending.
    rewrite_pending: bool,
    /// Lookup indices over `record.events` (seq -> position, uid -> seq), so
    /// the per-delivery conflict / re-echo checks are O(1) instead of a scan
    /// per event. Lookup-only: nothing iterates them, so `HashMap` order can
    /// never leak into behaviour. Verified on every hit (see [`Self::find`]),
    /// so a direct mutation of the public `record` degrades to a scan rather
    /// than a wrong answer.
    seq_index: std::collections::HashMap<u32, usize>,
    uid_index: std::collections::HashMap<u64, u32>,
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
        {
            Self::init_in(GAMES_DIR, seed)
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self::with_record(GameRecord {
                initial_state: InitialGameState { seed },
                events: Vec::new(),
            })
        }
    }

    /// A recorder holding `record` with no on-disk location.
    fn with_record(record: GameRecord) -> Self {
        let mut recorder = Self {
            record: Some(record),
            ..Default::default()
        };
        recorder.reindex();
        recorder
    }

    /// Native [`GameRecorder::init`] rooted at `games_dir` (tests use a
    /// temporary directory).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn init_in(games_dir: &str, seed: u64) -> Self {
        // Millisecond precision plus a per-process random suffix so two
        // local instances starting in the same second cannot land on the
        // same directory and interleave their appends into one corrupt
        // file (which produced doubled `}{ ` lines). Each instance records
        // to its own directory.
        let ts = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S-%3fZ");
        let suffix = format!("{:04x}", omdurman_net::new_seed() as u16);
        let dir = format!("{games_dir}/game_{ts}_{suffix}");
        if let Err(error) = std::fs::create_dir_all(&dir) {
            warn!(%error, %dir, "failed to create game directory");
        }
        let path = format!("{dir}/events.jsonl");
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
        let mut recorder = Self::with_record(GameRecord {
            initial_state: InitialGameState { seed },
            events: Vec::new(),
        });
        recorder.events_path = path;
        recorder.game_dir = dir;
        recorder
    }

    /// Path of this game's `events.jsonl` (native; empty before init).
    #[cfg(all(test, not(target_arch = "wasm32")))]
    pub(crate) fn events_path(&self) -> &str {
        &self.events_path
    }

    /// This game's artifact directory (`games/game_{ts}_{suffix}`), where the
    /// telegram / newspaper flavour text is persisted. `None` on wasm (no
    /// filesystem) or before [`GameRecorder::init`].
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
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
    /// (late-joiner / resync path). The on-disk log is rewritten from
    /// scratch on the next [`flush_game_record`] -- truncated, re-headed with
    /// the installed record's seed, then every installed event -- so it never
    /// holds the old line followed by a duplicate of the new one.
    pub(crate) fn install_history(&mut self, record: GameRecord) {
        self.record = Some(record);
        self.reindex();
        self.flushed_count = 0;
        self.rewrite_pending = true;
        self.dirty = true;
    }

    /// Same-room resync reset (stall / reconnect): empty the in-memory
    /// record -- the canonical one is re-downloaded -- but keep this game's
    /// directory, so a reconnect does not scatter one game over several
    /// `games/` entries. The file itself is left alone until something new
    /// is recorded (or a history installed), which then rewrites it whole.
    pub(crate) fn reset_for_resync(&mut self) {
        let seed = self
            .record
            .as_ref()
            .map_or_else(new_seed, |r| r.initial_state.seed);
        self.record = Some(GameRecord {
            initial_state: InitialGameState { seed },
            events: Vec::new(),
        });
        self.reindex();
        self.flushed_count = 0;
        self.rewrite_pending = true;
        self.dirty = false;
    }

    fn reindex(&mut self) {
        self.seq_index.clear();
        self.uid_index.clear();
        let Some(record) = &self.record else { return };
        for (idx, e) in record.events.iter().enumerate() {
            self.seq_index.entry(e.seq).or_insert(idx);
            if let Some(uid) = e.uid {
                self.uid_index.entry(uid).or_insert(e.seq);
            }
        }
    }

    /// Append `event` to the record, tagged with `sender_idx` and the
    /// canonical host-assigned `seq` (§ordering). Idempotent: a `seq` already
    /// present is ignored, guarding against duplicate delivery of a sequenced
    /// event. Returns `true` if the event was actually recorded (false if the
    /// recorder hasn't been initialised yet, or the `seq` was a duplicate).
    pub fn push_event(
        &mut self,
        event: &GameEvent,
        sender_idx: Option<u8>,
        seq: u32,
        uid: Option<u64>,
    ) -> bool {
        if self.record.is_none() || self.event_at_seq(seq).is_some() {
            return false;
        }
        let Some(record) = &mut self.record else {
            return false;
        };
        self.seq_index.insert(seq, record.events.len());
        if let Some(uid) = uid {
            self.uid_index.entry(uid).or_insert(seq);
        }
        record.events.push(RecordedEvent {
            utc: chrono::Utc::now(),
            sender_idx,
            seq,
            uid,
            payload: event.clone(),
        });
        self.dirty = true;
        true
    }

    /// Index-accelerated lookup of the first event matching `pred`. A hit is
    /// verified against the record; a miss is trusted only while the index
    /// covers every event (otherwise the public `record` was mutated
    /// directly and this falls back to a scan).
    fn find(
        &self,
        hint: Option<usize>,
        pred: impl Fn(&RecordedEvent) -> bool,
    ) -> Option<&RecordedEvent> {
        let events = &self.record.as_ref()?.events;
        if let Some(e) = hint.and_then(|i| events.get(i)).filter(|e| pred(e)) {
            return Some(e);
        }
        if hint.is_some() || self.seq_index.len() != events.len() {
            return events.iter().find(|e| pred(e));
        }
        None
    }

    /// The recorded event occupying `seq`, if any. Used by the receive path
    /// to detect seq conflicts (a delivery at an already-used seq carrying a
    /// *different* event), which prove the local record divergent.
    pub fn event_at_seq(&self, seq: u32) -> Option<&RecordedEvent> {
        self.find(self.seq_index.get(&seq).copied(), |e| e.seq == seq)
    }

    /// The seq under which the submission `uid` was recorded, if any. Used by
    /// the host to re-echo a retransmitted submission idempotently.
    pub fn seq_of_uid(&self, uid: u64) -> Option<u32> {
        let hint = self
            .uid_index
            .get(&uid)
            .and_then(|seq| self.seq_index.get(seq))
            .copied();
        self.find(hint, |e| e.uid == Some(uid)).map(|e| e.seq)
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

/// A record file that cannot be written (read-only directory, full disk):
/// which file, when to try again, and how many tries have failed in a row.
/// (Unused on the web, which writes no files.)
#[derive(Default)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct WriteRetry {
    path: String,
    next_try: Option<bevy::platform::time::Instant>,
    failures: u32,
}

impl WriteRetry {
    /// Whether a write may be tried now (not backing off).
    #[cfg(not(target_arch = "wasm32"))]
    fn due(&self) -> bool {
        self.next_try
            .is_none_or(|at| bevy::platform::time::Instant::now() >= at)
    }

    /// A failed write: back off (0.5 s, doubling, at most 30 s). Returns
    /// whether this is the first failure of the streak (worth a warning).
    #[cfg(not(target_arch = "wasm32"))]
    fn failed(&mut self) -> bool {
        self.failures += 1;
        let secs = (0.5 * 2f32.powi(self.failures.min(8) as i32 - 1)).min(30.0);
        self.next_try =
            Some(bevy::platform::time::Instant::now() + std::time::Duration::from_secs_f32(secs));
        self.failures == 1
    }

    /// A successful write ends the streak. Returns whether one was running.
    #[cfg(not(target_arch = "wasm32"))]
    fn succeeded(&mut self) -> bool {
        let recovered = self.failures > 0;
        self.next_try = None;
        self.failures = 0;
        recovered
    }

    /// Writes now go to `path`: a streak against another file (the previous
    /// game's) does not hold this one back.
    #[cfg(not(target_arch = "wasm32"))]
    fn target(&mut self, path: &str) {
        if self.path != path {
            *self = Self {
                path: path.to_owned(),
                ..Self::default()
            };
        }
    }
}

/// Append unreleased events to the JSONL file (native only).
/// On WASM, keep everything in memory; user can download via a button.
pub fn flush_game_record(mut recorder: ResMut<GameRecorder>, mut retry: Local<WriteRetry>) {
    if !recorder.dirty {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        // On WASM everything stays in memory; the user downloads it via a
        // button, so there's nothing to write -- just clear the flag.
        recorder.dirty = false;
        let _ = &mut retry;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let recorder = &mut *recorder;
        let Some(ref record) = recorder.record else {
            recorder.dirty = false;
            return;
        };
        if recorder.events_path.is_empty() {
            // No on-disk location (never initialised on native).
            recorder.dirty = false;
            return;
        }
        use std::io::Write;
        let rewrite = recorder.rewrite_pending;
        let start = if rewrite { 0 } else { recorder.flushed_count };
        let new_events = &record.events[start.min(record.events.len())..];
        if new_events.is_empty() && !rewrite {
            recorder.dirty = false;
            return;
        }
        // `dirty` stays set until the write succeeds, so a failed open or
        // write is retried rather than silently dropping the events -- with
        // a backoff, warned once per failing streak: a directory that is not
        // writable must not warn every frame for the rest of the session. A
        // rewrite truncates and re-heads the file; an append extends it.
        retry.target(&recorder.events_path);
        if !retry.due() {
            return;
        }
        let opened = if rewrite {
            std::fs::File::create(&recorder.events_path).and_then(|mut f| {
                writeln!(f, r#"{{"seed":{}}}"#, record.initial_state.seed).map(|()| f)
            })
        } else {
            std::fs::OpenOptions::new()
                .append(true)
                .open(&recorder.events_path)
        };
        let mut f = match opened {
            Ok(f) => f,
            Err(error) => {
                if retry.failed() {
                    warn!(%error, path = %recorder.events_path, rewrite, "failed to open game record; will retry");
                } else {
                    debug!(%error, path = %recorder.events_path, "game record still not writable");
                }
                return;
            }
        };
        let mut all_written = true;
        for ev in new_events {
            let Ok(line) = serde_json::to_string(ev)
                .inspect_err(|error| warn!(%error, "failed to serialise recorded event; skipping"))
            else {
                continue;
            };
            if let Err(error) = writeln!(f, "{line}") {
                if retry.failed() {
                    warn!(%error, path = %recorder.events_path, "failed to write recorded event; will retry");
                } else {
                    debug!(%error, path = %recorder.events_path, "game record still not writable");
                }
                all_written = false;
                break;
            }
        }
        if all_written {
            if retry.succeeded() {
                info!(path = %recorder.events_path, "game record writable again");
            }
            recorder.flushed_count = record.events.len();
            recorder.rewrite_pending = false;
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
    let SeedHeader { seed } =
        serde_json::from_str(header).map_err(|source| LoadRecordError::SeedHeader {
            path: path.to_string(),
            source,
        })?;
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
        initial_state: InitialGameState { seed },
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
            events
                .is_file()
                .then(|| Some((events.to_str()?.to_string(), name)))?
        })
        .collect();
    // Names embed a sortable UTC timestamp, so a reverse sort on it puts the
    // newest game first.
    games.sort_by(|a, b| saved_game_timestamp(&b.1).cmp(saved_game_timestamp(&a.1)));
    games
}

/// The sortable UTC timestamp in a saved game's directory name: `game_<ts>`
/// from the app, `game_bot_<ts>` from the bot CLI. Sorting on the whole name
/// put every bot game ahead of every human one ('b' sorts after the digits).
#[cfg(not(target_arch = "wasm32"))]
fn saved_game_timestamp(name: &str) -> &str {
    let rest = name.strip_prefix("game_").unwrap_or(name);
    rest.strip_prefix("bot_").unwrap_or(rest)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn saved_games_sort_by_time_whoever_played() {
        let mut names = [
            "game_bot_2026-09-03T15-40-23-625Z",
            "game_2026-09-28T00-33-56-017Z_c622",
            "game_2026-09-01T10-00-00-000Z_abcd",
        ];
        names.sort_by(|a, b| saved_game_timestamp(b).cmp(saved_game_timestamp(a)));
        assert_eq!(
            names,
            [
                "game_2026-09-28T00-33-56-017Z_c622",
                "game_bot_2026-09-03T15-40-23-625Z",
                "game_2026-09-01T10-00-00-000Z_abcd",
            ]
        );
    }

    fn event(n: u32) -> GameEvent {
        GameEvent::RemoveUnit {
            sprite: omdurman_types::SpriteRef {
                section_name: omdurman_types::SectionName::Taiasha,
                col: n,
                row: 0,
            },
        }
    }

    fn recorded(seq: u32) -> RecordedEvent {
        RecordedEvent {
            utc: chrono::Utc::now(),
            sender_idx: None,
            seq,
            uid: Some(u64::from(seq) + 100),
            payload: event(seq),
        }
    }

    fn flush(app: &mut App) {
        app.world_mut()
            .run_system_once(flush_game_record)
            .expect("flush runs");
    }

    fn on_disk(app: &App) -> GameRecord {
        let path = app
            .world()
            .resource::<GameRecorder>()
            .events_path()
            .to_owned();
        load_record_from_jsonl(&path).expect("record file parses")
    }

    // D4: installing a history rewrites the log (truncate + the installed
    // record's seed header + its events) instead of appending the whole
    // record after the old one.
    #[test]
    fn install_history_rewrites_log_without_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let mut recorder = GameRecorder::init_in(dir.path().to_str().unwrap(), 11);
        recorder.push_event(&event(0), None, 0, Some(100));
        recorder.push_event(&event(1), None, 1, Some(101));
        let mut app = App::new();
        app.insert_resource(recorder);
        flush(&mut app);
        assert_eq!(on_disk(&app).events.len(), 2);

        app.world_mut()
            .resource_mut::<GameRecorder>()
            .install_history(GameRecord {
                initial_state: InitialGameState { seed: 42 },
                events: (0..3).map(recorded).collect(),
            });
        flush(&mut app);
        let disk = on_disk(&app);
        assert_eq!(disk.initial_state.seed, 42, "header carries installed seed");
        let seqs: Vec<u32> = disk.events.iter().map(|e| e.seq).collect();
        assert_eq!(seqs, vec![0, 1, 2], "no duplicated seqs");

        // Later live events append after the rewritten log.
        app.world_mut()
            .resource_mut::<GameRecorder>()
            .push_event(&event(3), None, 3, Some(103));
        flush(&mut app);
        let seqs: Vec<u32> = on_disk(&app).events.iter().map(|e| e.seq).collect();
        assert_eq!(seqs, vec![0, 1, 2, 3]);
    }

    // D4: a same-room resync reset keeps the directory; the next recorded
    // event rewrites the file instead of appending a second seq line.
    #[test]
    fn resync_reset_rewrites_on_next_event() {
        let dir = tempfile::tempdir().unwrap();
        let mut recorder = GameRecorder::init_in(dir.path().to_str().unwrap(), 3);
        recorder.push_event(&event(0), None, 0, Some(100));
        let mut app = App::new();
        app.insert_resource(recorder);
        flush(&mut app);

        let game_dir = app.world().resource::<GameRecorder>().artifacts_dir();
        app.world_mut()
            .resource_mut::<GameRecorder>()
            .reset_for_resync();
        assert_eq!(
            app.world().resource::<GameRecorder>().artifacts_dir(),
            game_dir
        );
        flush(&mut app);
        assert_eq!(on_disk(&app).events.len(), 1, "untouched until new data");
        app.world_mut()
            .resource_mut::<GameRecorder>()
            .push_event(&event(5), None, 0, Some(105));
        flush(&mut app);
        let disk = on_disk(&app);
        assert_eq!(disk.initial_state.seed, 3);
        assert_eq!(disk.events.len(), 1);
        assert_eq!(disk.events[0].payload, event(5));
    }

    // D9: the seq / uid indices agree with the record.
    #[test]
    fn lookups_by_seq_and_uid() {
        let mut recorder = GameRecorder::default();
        recorder.install_history(GameRecord {
            initial_state: InitialGameState { seed: 1 },
            events: vec![recorded(4), recorded(7)],
        });
        recorder.push_event(&event(9), None, 9, Some(900));
        assert!(
            !recorder.push_event(&event(9), None, 9, Some(901)),
            "seq dedup"
        );
        assert_eq!(recorder.event_at_seq(7).map(|e| e.seq), Some(7));
        assert!(recorder.event_at_seq(5).is_none());
        assert_eq!(recorder.seq_of_uid(104), Some(4));
        assert_eq!(recorder.seq_of_uid(900), Some(9));
        assert_eq!(recorder.seq_of_uid(901), None);
        // A direct mutation of the public record degrades to a scan.
        recorder.record.as_mut().unwrap().events.push(recorded(12));
        assert_eq!(recorder.event_at_seq(12).map(|e| e.seq), Some(12));
        assert_eq!(recorder.seq_of_uid(112), Some(12));
    }
}

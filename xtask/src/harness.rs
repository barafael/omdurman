//! A client of a peer's agent harness (`omdurman-app/src/agent.rs`; its protocol is the
//! `omdurman-harness` crate).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use omdurman_harness::{Acted, Action, Call, Reply, Request, Snapshot};
use serde::de::DeserializeOwned;

use crate::Result;

/// How often [`Harness::wait_for`] looks again: the harness answers on the game's frames.
const POLL: Duration = Duration::from_millis(500);
/// How long a reply may take: a frame at the latest, which an idle game paces at 500 ms.
const REPLY: Duration = Duration::from_secs(30);

pub struct Harness {
    /// The peer's name, for messages.
    pub peer: &'static str,
    stream: UnixStream,
    lines: BufReader<UnixStream>,
    next_id: u64,
    /// Where the peer's `/run/omdurman` is on the host: its screenshots land there.
    run_dir: PathBuf,
}

impl Harness {
    /// Connects to `peer`'s socket in `run_dir`; the caller waits for the peer to say it listens.
    pub fn connect(peer: &'static str, run_dir: &Path) -> Result<Self> {
        let socket = run_dir.join("agent.sock");
        let stream = UnixStream::connect(&socket)
            .map_err(|error| format!("{}: {error}", socket.display()))?;
        stream
            .set_read_timeout(Some(REPLY))
            .map_err(|error| format!("{peer}: {error}"))?;
        let lines = BufReader::new(
            stream
                .try_clone()
                .map_err(|error| format!("{peer}: {error}"))?,
        );
        Ok(Self {
            peer,
            stream,
            lines,
            next_id: 1,
            run_dir: run_dir.to_owned(),
        })
    }

    fn request<T: DeserializeOwned>(&mut self, call: Call) -> Result<T> {
        let id = self.next_id;
        self.next_id += 1;
        let line =
            serde_json::to_string(&Request { id, call }).map_err(|error| error.to_string())?;
        writeln!(self.stream, "{line}")
            .map_err(|error| format!("{}: sending: {error}", self.peer))?;
        let mut line = String::new();
        if self
            .lines
            .read_line(&mut line)
            .map_err(|error| format!("{}: reading a reply: {error}", self.peer))?
            == 0
        {
            return Err(format!("{}: the harness closed the connection", self.peer));
        }
        let reply: Reply<T> = serde_json::from_str(&line)
            .map_err(|error| format!("{}: {error}: {line}", self.peer))?;
        if reply.id != id {
            return Err(format!("{}: a reply to {}, not {id}", self.peer, reply.id));
        }
        Result::from(reply.outcome).map_err(|error| format!("{}: {error}", self.peer))
    }

    pub fn snapshot(&mut self) -> Result<Snapshot> {
        self.request(Call::Snapshot)
    }

    pub fn act(&mut self, action: Action) -> Result<()> {
        self.request::<Acted>(Call::Act { action }).map(drop)
    }

    /// The window as drawn, as a PNG on the host, once the game has written it.
    pub fn screenshot(&mut self) -> Result<PathBuf> {
        let on_host = self.run_dir.join("shot.png");
        match std::fs::remove_file(&on_host) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{}: {error}", on_host.display())),
        }
        self.request::<Acted>(Call::Screenshot {
            path: "/run/omdurman/shot.png".into(),
        })?;
        // Rendered, read back and encoded some frames later; whole once its size holds.
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut size = None;
        loop {
            std::thread::sleep(POLL);
            let now = std::fs::metadata(&on_host).ok().map(|found| found.len());
            if now.is_some_and(|now| now > 0) && now == size {
                return Ok(on_host);
            }
            size = now;
            if Instant::now() > deadline {
                return Err(format!("{}: no screenshot within 20 s", self.peer));
            }
        }
    }

    /// Waits until `accept` returns a value for what the peer holds, or fails after `patience`
    /// with `waiting_for` and the last snapshot's gist.
    pub fn wait_for<Found>(
        &mut self,
        waiting_for: &str,
        patience: Duration,
        mut accept: impl FnMut(&Snapshot) -> Option<Found>,
    ) -> Result<Found> {
        let deadline = Instant::now() + patience;
        loop {
            let snapshot = self.snapshot()?;
            if let Some(found) = accept(&snapshot) {
                return Ok(found);
            }
            if Instant::now() > deadline {
                return Err(format!(
                    "{}: {}s waiting for {waiting_for}; {}",
                    self.peer,
                    patience.as_secs(),
                    gist(&snapshot)
                ));
            }
            std::thread::sleep(POLL);
        }
    }
}

/// What a failure says of a peer's state.
pub fn gist(snapshot: &Snapshot) -> String {
    let net = &snapshot.net;
    let game = &snapshot.game;
    format!(
        "screen {:?}, state {}, {} peer(s), host {}, seq {:?}, {} event(s) {}, {} unconfirmed; \
         game started {}, turn {} phase {} to act {:?}, paused {}, units {}",
        snapshot.screen,
        snapshot.app_state,
        net.peers.len(),
        net.is_host,
        net.last_applied_seq,
        net.record_len,
        net.record_digest,
        net.unconfirmed,
        game.started,
        game.turn,
        game.phase,
        game.to_act,
        game.paused,
        game.units_digest,
    )
}

//! Native persistence of the local [`PlayerKey`], so a player who quits or
//! crashes and relaunches the app is bound to their seat again (the web build
//! keeps the key in `sessionStorage` instead, see `seats::LocalPlayerKey`).
//!
//! Keys live in numbered *slot* files (`player_key_0`, `player_key_1`, …) in
//! the user's config directory. Each running instance takes the first slot
//! whose file it can lock exclusively and holds that lock for the life of the
//! process; the OS releases it when the process exits or dies. So:
//! * a relaunch finds its old slot free again and reuses the key;
//! * a second window running beside it takes the next slot — a distinct
//!   player, which keeps local multi-window testing working.
//!
//! `OMDURMAN_PLAYER_SLOT=<n>` pins an instance to one slot (e.g. to make a
//! test window always come back as the same player).

use omdurman_net::PlayerKey;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Upper bound on concurrently running instances that get a persistent key;
/// beyond it an instance falls back to a fresh per-process key.
const MAX_SLOTS: u32 = 16;

/// The locked slot file, held until the process exits (dropping it would
/// release the lock and let another instance take this identity).
#[cfg_attr(test, allow(dead_code))] // Tests use `claim_slot` on a temp dir.
static HELD_SLOT: OnceLock<File> = OnceLock::new();

/// The persisted key of the first free slot, or a fresh key if no config
/// directory is usable or every slot is taken.
#[cfg_attr(test, allow(dead_code))] // Tests use `claim_slot` on a temp dir.
pub(crate) fn load_or_create() -> PlayerKey {
    let pinned = std::env::var("OMDURMAN_PLAYER_SLOT")
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok());
    let claimed = config_dir().and_then(|dir| claim_slot(&dir, pinned));
    match claimed {
        Some((key, file)) => {
            // First call wins; the resource is only built once at startup.
            let _ = HELD_SLOT.set(file);
            key
        }
        None => PlayerKey::random(),
    }
}

/// The per-user config directory for the game: `$XDG_CONFIG_HOME` or
/// `~/.config` on Linux, `~/Library/Application Support` on macOS,
/// `%APPDATA%` on Windows — each with an `omdurman` subdirectory.
#[cfg_attr(test, allow(dead_code))] // Tests use `claim_slot` on a temp dir.
fn config_dir() -> Option<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }?;
    Some(base.join("omdurman"))
}

/// Lock the first free slot in `dir` (or only `pinned`) and return its key —
/// read from the file, or freshly generated and written when the file is new
/// or unreadable — together with the locked file.
fn claim_slot(dir: &Path, pinned: Option<u32>) -> Option<(PlayerKey, File)> {
    std::fs::create_dir_all(dir).ok()?;
    let slots = match pinned {
        Some(slot) => slot..slot + 1,
        None => 0..MAX_SLOTS,
    };
    for slot in slots {
        let path = dir.join(format!("player_key_{slot}"));
        let Ok(mut file) = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
        else {
            continue;
        };
        if file.try_lock().is_err() {
            continue; // Another running instance holds this identity.
        }
        let mut stored = String::new();
        let existing = file
            .read_to_string(&mut stored)
            .ok()
            .and_then(|_| u64::from_str_radix(stored.trim(), 16).ok())
            .map(PlayerKey);
        let key = match existing {
            Some(key) => key,
            None => {
                let key = PlayerKey::random();
                let written = file
                    .set_len(0)
                    .and_then(|()| file.seek(SeekFrom::Start(0)))
                    .and_then(|_| writeln!(file, "{key}"));
                if let Err(error) = written {
                    bevy::log::warn!(%error, ?path, "could not persist the player key");
                }
                key
            }
        };
        return Some((key, file));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::claim_slot;

    #[test]
    fn relaunch_reuses_the_key_and_concurrent_instances_get_distinct_ones() {
        let dir = tempfile::tempdir().unwrap();
        let (first, first_lock) = claim_slot(dir.path(), None).expect("slot 0");
        // A second instance running at the same time must not share it.
        let (second, _second_lock) = claim_slot(dir.path(), None).expect("slot 1");
        assert_ne!(first, second);
        // The first instance exits (lock released); a relaunch gets its key back.
        drop(first_lock);
        let (relaunched, _lock) = claim_slot(dir.path(), None).expect("slot 0 again");
        assert_eq!(relaunched, first);
    }

    #[test]
    fn pinned_slot_is_exclusive() {
        let dir = tempfile::tempdir().unwrap();
        let (key, _lock) = claim_slot(dir.path(), Some(3)).expect("slot 3");
        assert!(
            claim_slot(dir.path(), Some(3)).is_none(),
            "a pinned slot in use is not shared"
        );
        let (other, _) = claim_slot(dir.path(), None).expect("free slot");
        assert_ne!(key, other);
    }

    #[test]
    fn corrupt_slot_file_is_replaced_with_a_fresh_key() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("player_key_0"), "not hex").unwrap();
        let (key, lock) = claim_slot(dir.path(), None).expect("slot 0");
        drop(lock);
        let (again, _) = claim_slot(dir.path(), None).expect("slot 0");
        assert_eq!(key, again, "the replacement key was persisted");
    }
}

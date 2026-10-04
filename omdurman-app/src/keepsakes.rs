//! Keepsakes of the game played: on native, a screenshot of every turn's
//! field telegram when it is first shown, and of the newspaper's front page
//! at game over, saved in the working directory and named after the game's
//! record (`game_<time>_<id>-telegram-turn-03.png`, `...-gazette.png`).
//! Only live play is kept -- reviewing a record takes no pictures -- and
//! each picture is taken once.

use bevy::prelude::*;

/// Pictures asked for and taken this session.
#[derive(Resource, Default)]
pub(crate) struct Keepsakes {
    /// File names already taken (or queued): each is taken once.
    taken: std::collections::BTreeSet<String>,
    /// Pictures to take, with the seconds still to wait so the screen they
    /// show has faded in (egui fades a new area in over a fraction of a
    /// second; an early picture caught an empty, translucent card).
    queue: std::collections::VecDeque<(String, f32)>,
}

/// Seconds between a screen first showing and its picture.
const SETTLE_SECS: f32 = 0.6;

impl Keepsakes {
    /// Ask for a picture of what is on screen now, called `what` (e.g.
    /// "telegram-turn-03"), for the game whose record is in `recorder`.
    /// Ignored outside a recorded game, on the web, and when already taken.
    pub(crate) fn request(&mut self, recorder: &crate::game_record::GameRecorder, what: &str) {
        let Some(game) = recorder.artifacts_dir().and_then(|dir| {
            std::path::Path::new(&dir)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
        }) else {
            return;
        };
        let name = format!("{game}-{what}.png");
        if self.taken.insert(name.clone()) {
            self.queue.push_back((name, SETTLE_SECS));
        }
    }
}

/// Take the queued pictures, one at a time, once their screen has settled.
#[cfg_attr(target_arch = "wasm32", allow(unused_variables, unused_mut))]
pub(crate) fn take_keepsakes(
    mut commands: Commands,
    mut keepsakes: ResMut<Keepsakes>,
    mut activity: ResMut<crate::activity::Activity>,
    time: Res<Time>,
) {
    let Some((_, wait)) = keepsakes.queue.front_mut() else {
        return;
    };
    // The countdown needs frames: the app otherwise idles between inputs.
    activity.keep_running();
    if *wait > 0.0 {
        *wait -= time.delta_secs();
        return;
    }
    let Some((name, _)) = keepsakes.queue.pop_front() else {
        return;
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        use bevy::render::view::screenshot::{Screenshot, save_to_disk};
        info!(%name, "keepsake screenshot");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(name));
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn each_keepsake_is_named_after_the_game_and_taken_once() {
        let games = tempfile::tempdir().expect("temp dir");
        let recorder = crate::game_record::GameRecorder::init_in(
            games.path().to_str().expect("utf-8 temp path"),
            7,
        );
        let game = std::path::Path::new(&recorder.artifacts_dir().expect("a game dir"))
            .file_name()
            .expect("a dir name")
            .to_string_lossy()
            .into_owned();
        let mut keepsakes = Keepsakes::default();
        keepsakes.request(&recorder, "telegram-turn-01");
        keepsakes.request(&recorder, "telegram-turn-01");
        keepsakes.request(&recorder, "gazette");
        let names: Vec<&str> = keepsakes.queue.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            [
                format!("{game}-telegram-turn-01.png"),
                format!("{game}-gazette.png")
            ]
        );
        // Outside a recorded game nothing is asked for.
        let mut idle = Keepsakes::default();
        idle.request(&crate::game_record::GameRecorder::default(), "gazette");
        assert!(idle.queue.is_empty());
    }
}

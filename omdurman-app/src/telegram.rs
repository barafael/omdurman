use bevy::prelude::*;

use omdurman_net::GameEvent;

/// The field telegrams: one per finished turn, written by the host from the
/// turn's record ([`omdurman_rules::press::telegram`]) and filed from the
/// recorded [`GameEvent::Telegram`], so every peer -- and a replay -- reads
/// the same text. (The end-of-game newspaper is composed by every peer from
/// the game state, see `crate::newspaper`; a [`GameEvent::Gazette`] in an
/// older record is filed but no longer shown.)
#[derive(Resource, Default)]
pub struct TelegramLog {
    /// Filed telegrams, one per turn, in record order.
    pub entries: Vec<(u8, String)>,
    /// Host: turns whose telegram has been written and submitted.
    pub requested: std::collections::BTreeSet<u8>,
    /// How many entries have been persisted to the artifacts file. The file
    /// is rewritten whole (sorted by turn) each time this lags behind
    /// `entries.len()` — entries arrive in completion order, not turn order.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub flushed: usize,
    /// How many `entries` the player has seen: the overlay shows the next
    /// unseen telegram of the latest turn and waits for a click (see
    /// `ui_plugin::controls::telegram_overlay`).
    pub acknowledged: usize,
    /// The Gazette paragraphs of an older record (no longer written).
    pub gazette: Option<Vec<String>>,
}

impl TelegramLog {
    /// Whether the latest finished turn's telegram still waits for this
    /// player: not yet filed (the host is writing it) or not yet dismissed.
    /// The host's AI holds its moves meanwhile, so play really waits for
    /// the overlay (`ui_plugin::controls::telegram_overlay`); at game over
    /// the newspaper replaces the telegram and nothing waits.
    pub(crate) fn awaiting_ack(&self, state: &omdurman_rules::effects::GameState) -> bool {
        if state.game_over {
            return false;
        }
        let Some(latest) = state.turn_summaries.last().map(|s| s.turn.value()) else {
            return false;
        };
        !self
            .entries
            .iter()
            .take(self.acknowledged)
            .any(|(turn, _)| *turn == latest)
    }

    /// File a press event (the `game_apply` arm for both variants): the
    /// first telegram recorded for a turn wins, as does the first Gazette.
    /// Returns whether it was filed.
    pub(crate) fn file(&mut self, event: &GameEvent) -> bool {
        match event {
            GameEvent::Telegram { turn, text } => {
                if self.entries.iter().any(|(t, _)| t == turn) {
                    return false;
                }
                self.entries.push((*turn, text.clone()));
                true
            }
            GameEvent::Gazette { paragraphs } => {
                if self.gazette.is_some() {
                    return false;
                }
                self.gazette = Some(paragraphs.clone());
                true
            }
            _ => false,
        }
    }
}

/// Who writes the press, and where a written text goes. In a network
/// session only the host writes, and submits the text as a recorded event
/// (its echo files it on every peer, host included); with no session at all
/// (headless fixtures) the text is filed directly.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PressDesk<'w> {
    net: Option<Res<'w, omdurman_net::NetState>>,
    edits: Option<ResMut<'w, crate::PendingEdits>>,
}

impl PressDesk<'_> {
    /// Whether this peer writes the press.
    pub(crate) fn writes(&self) -> bool {
        self.net.as_ref().is_none_or(|net| net.is_host)
    }

    /// Send a written press event on its way (see [`PressDesk`]).
    pub(crate) fn publish(&mut self, log: &mut TelegramLog, event: GameEvent) {
        match self.edits.as_mut() {
            Some(edits) if self.net.is_some() => {
                edits.submit_game(event);
            }
            _ => {
                log.file(&event);
            }
        }
    }
}

/// Host: write the telegram of every finished turn that has none yet, from
/// the turn's record and the game as the turn left it, and submit it.
/// Guests only wait for the host's telegram to arrive. A new host picks up
/// any turn the old one left unwritten.
pub(crate) fn generate_telegrams(
    game_state: Option<Res<crate::GameStateResource>>,
    mut telegram_log: ResMut<TelegramLog>,
    mut desk: PressDesk,
) {
    let Some(state) = game_state else { return };
    if !desk.writes() {
        return;
    }
    for summary in &state.0.turn_summaries {
        let turn = summary.turn.value();
        if telegram_log.requested.contains(&turn)
            || telegram_log.entries.iter().any(|(t, _)| *t == turn)
        {
            continue;
        }
        telegram_log.requested.insert(turn);
        let text = omdurman_rules::press::telegram::telegram(&state.0, summary);
        desk.publish(&mut telegram_log, GameEvent::Telegram { turn, text });
    }
}

/// Persist new telegram entries to the game's artifact directory
/// (`games/<game>/telegrams.md`), native only. No-op on wasm.
#[cfg_attr(target_arch = "wasm32", allow(unused_variables, unused_mut))]
pub(crate) fn save_telegram_artifacts(
    recorder: Res<crate::game_record::GameRecorder>,
    mut telegram_log: ResMut<TelegramLog>,
    mut retry: Local<crate::game_record::WriteRetry>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if telegram_log.entries.len() <= telegram_log.flushed {
            return;
        }
        let Some(dir) = recorder.artifacts_dir() else {
            return;
        };
        let path = format!("{dir}/telegrams.md");
        // A file that cannot be written is retried with a backoff and warned
        // about once per failing streak (see `WriteRetry`), not rewritten
        // and warned about every frame for the rest of the session.
        retry.target(&path);
        if !retry.due() {
            return;
        }
        let mut sorted = telegram_log.entries.clone();
        sorted.sort_by_key(|(turn, _)| *turn);
        let result = (|| -> std::io::Result<()> {
            use std::io::Write;
            let mut f = std::fs::File::create(&path)?;
            writeln!(f, "# Military telegrams")?;
            writeln!(f)?;
            for (turn, text) in &sorted {
                writeln!(f, "## Turn {turn}")?;
                writeln!(f, "{text}")?;
                writeln!(f)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                if retry.succeeded() {
                    info!(%path, "telegrams artifact writable again");
                }
                telegram_log.flushed = telegram_log.entries.len();
            }
            // Leave `flushed` behind so the write is retried (after the backoff).
            Err(error) => {
                if retry.failed() {
                    warn!(%error, %path, "failed to write telegrams artifact; will retry");
                } else {
                    debug!(%error, %path, "telegrams artifact still not writable");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TelegramLog;
    use omdurman_rules::effects::GameState;
    use omdurman_rules::turn_summary::TurnSummary;
    use omdurman_types::Scenario;

    fn finished_turn(state: &mut GameState, turn: u8) {
        state.turn_summaries.push(TurnSummary {
            turn: omdurman_rules::GameTurnIndex::new(turn),
            time: omdurman_rules::turn_track::GameTime::SixAM,
            day_night: state.day_night,
            first_player: omdurman_rules::effects::first_player(state.scenario),
            events: Vec::new(),
        });
    }

    /// The host's AI holds its moves while the latest turn's telegram is
    /// unfiled or unread, and not once it has been dismissed.
    #[test]
    fn the_latest_telegram_holds_play_until_dismissed() {
        let mut state = GameState::new(Scenario::Campaign);
        let mut log = TelegramLog::default();
        assert!(!log.awaiting_ack(&state), "no turn finished yet");

        finished_turn(&mut state, 1);
        assert!(
            log.awaiting_ack(&state),
            "the telegram is still being written"
        );
        log.entries.push((1, "turn 1".into()));
        assert!(log.awaiting_ack(&state), "filed but unread");
        log.acknowledged = 1;
        assert!(!log.awaiting_ack(&state), "read");

        finished_turn(&mut state, 2);
        assert!(
            log.awaiting_ack(&state),
            "the next turn's telegram waits again"
        );
        state.game_over = true;
        assert!(
            !log.awaiting_ack(&state),
            "at game over the newspaper takes over"
        );
    }
}

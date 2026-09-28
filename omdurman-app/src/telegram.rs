use bevy::prelude::*;
use bevy::tasks::futures::check_ready;

use omdurman_net::GameEvent;

use crate::llm::{CompletionTag, LlmConfig, PendingCompletions, spawn_completion};

/// The press of the game: one field telegram per finished turn and, at game
/// over, the Gazette's report. Filed from recorded events
/// ([`GameEvent::Telegram`] / [`GameEvent::Gazette`]), so every peer -- and a
/// replay -- reads the same text; only the host writes it.
#[derive(Resource, Default)]
pub struct TelegramLog {
    /// Filed telegrams, one per turn, in record order.
    pub entries: Vec<(u8, String)>,
    /// Host: turns whose telegram has been asked for (the model's answer is
    /// in flight, or it has been submitted).
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
    /// Per-turn fallback text built from the turn's own events, used when
    /// no flavour model is configured or its request fails -- never an
    /// empty "the situation develops" stub.
    pub fallbacks: std::collections::HashMap<u8, String>,
    /// The Gazette's report paragraphs, once filed.
    pub gazette: Option<Vec<String>>,
}

impl TelegramLog {
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
    /// Whether this peer writes the press (so asks the flavour model).
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

/// A telegram written from the turn's recorded events, for when no flavour
/// model is available: the first few dispatch lines, or a quiet-turn line.
pub(crate) fn fallback_telegram(summary: &omdurman_rules::turn_summary::TurnSummary) -> String {
    let lines: Vec<String> = summary
        .events
        .iter()
        .filter(|e| {
            !matches!(
                e,
                omdurman_rules::turn_summary::TurnEventRecord::VpScored { .. }
            )
        })
        .take(5)
        .map(|e| e.format_for_dispatch())
        .collect();
    if lines.is_empty() {
        "All quiet. The lines held; no engagements to report.".to_string()
    } else {
        lines.join(". ") + "."
    }
}

/// Host: write the telegram of every finished turn that has none yet -- ask
/// the flavour model, or publish the turn's own events at once when there is
/// none. Guests only wait for the host's telegram to arrive. A new host picks
/// up any turn the old one left unwritten.
pub(crate) fn generate_telegrams(
    game_state: Option<Res<crate::GameStateResource>>,
    llm_config: Res<LlmConfig>,
    mut telegram_log: ResMut<TelegramLog>,
    mut pending: ResMut<PendingCompletions>,
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
        let fallback = fallback_telegram(summary);
        telegram_log.fallbacks.insert(turn, fallback.clone());
        if llm_config.has_key() {
            let (system, user) =
                omdurman_rules::telegram_prompt::build_telegram_prompt(summary, state.0.scenario);
            spawn_completion(
                &llm_config,
                &system,
                &user,
                CompletionTag::Telegram { turn },
                crate::llm::TELEGRAM_MAX_TOKENS,
                &mut pending,
            );
        } else {
            desk.publish(
                &mut telegram_log,
                GameEvent::Telegram {
                    turn,
                    text: fallback,
                },
            );
        }
    }
}

/// Host: publish each telegram the flavour model returns (or, if it failed,
/// the turn's fallback text).
pub(crate) fn poll_telegram_completions(
    mut pending: ResMut<PendingCompletions>,
    mut telegram_log: ResMut<TelegramLog>,
    mut desk: PressDesk,
) {
    let mut i = 0;
    while i < pending.items.len() {
        if matches!(pending.items[i].tag, CompletionTag::Telegram { .. }) {
            if let Some(result) = check_ready(&mut pending.items[i].task) {
                let item = pending.items.swap_remove(i);
                match item.tag {
                    CompletionTag::Telegram { turn } => {
                        let text = result.unwrap_or_else(|e| {
                            warn!("LLM telegram generation failed for turn {turn}: {e}");
                            telegram_log
                                .fallbacks
                                .get(&turn)
                                .cloned()
                                .unwrap_or_else(|| stub_telegram_text(turn))
                        });
                        desk.publish(&mut telegram_log, GameEvent::Telegram { turn, text });
                    }
                    CompletionTag::Newspaper => unreachable!(),
                }
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
}

fn stub_telegram_text(turn: u8) -> String {
    format!(
        "[Turn {turn}] The situation develops. Our correspondent reports from the forward positions."
    )
}

/// Persist new telegram entries to the game's artifact directory
/// (`games/<game>/telegrams.md`), native only. No-op on wasm.
#[cfg_attr(target_arch = "wasm32", allow(unused_variables, unused_mut))]
pub(crate) fn save_telegram_artifacts(
    recorder: Res<crate::game_record::GameRecorder>,
    mut telegram_log: ResMut<TelegramLog>,
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
            Ok(()) => telegram_log.flushed = telegram_log.entries.len(),
            // Leave `flushed` behind so the write is retried next frame.
            Err(error) => warn!(%error, %path, "failed to write telegrams artifact"),
        }
    }
}

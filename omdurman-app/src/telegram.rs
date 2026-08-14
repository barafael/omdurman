use bevy::prelude::*;
use bevy::tasks::futures::check_ready;

use crate::llm::{CompletionTag, LlmConfig, PendingCompletions, spawn_completion};

#[derive(Resource, Default)]
pub struct TelegramLog {
    pub entries: Vec<(u8, String)>,
    pub last_processed: usize,
    pub pending_stubs: Vec<u8>,
    /// How many entries have been persisted to the artifacts file. The file
    /// is rewritten whole (sorted by turn) each time this lags behind
    /// `entries.len()` — entries arrive in completion order, not turn order.
    pub flushed: usize,
}

pub(crate) fn generate_telegrams(
    game_state: Option<Res<crate::GameStateResource>>,
    llm_config: Res<LlmConfig>,
    mut telegram_log: ResMut<TelegramLog>,
    mut pending: ResMut<PendingCompletions>,
) {
    let Some(state) = game_state else { return };
    let summaries = &state.0.turn_summaries;
    let len = summaries.len();
    if len <= telegram_log.last_processed {
        return;
    }
    for summary in &summaries[telegram_log.last_processed..] {
        let turn = summary.turn.value();
        if llm_config.has_key() {
            let (system, user) =
                omdurman_rules::telegram_prompt::build_telegram_prompt(summary);
            spawn_completion(
                &llm_config,
                &system,
                &user,
                CompletionTag::Telegram { turn },
                &mut pending,
            );
        } else {
            telegram_log.pending_stubs.push(turn);
        }
    }
    telegram_log.last_processed = len;
}

pub(crate) fn poll_telegram_completions(
    mut pending: ResMut<PendingCompletions>,
    mut telegram_log: ResMut<TelegramLog>,
) {
    let mut i = 0;
    while i < pending.items.len() {
        if matches!(pending.items[i].tag, CompletionTag::Telegram { .. }) {
            if let Some(result) = check_ready(&mut pending.items[i].task) {
                let item = pending.items.swap_remove(i);
                match item.tag {
                    CompletionTag::Telegram { turn } => {
                        let text =
                            result.unwrap_or_else(|e| stub_telegram_text(turn, e));
                        telegram_log.entries.push((turn, text));
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

    let stubs: Vec<u8> = std::mem::take(&mut telegram_log.pending_stubs);
    for turn in stubs {
        let text = format!(
            "[Turn {turn}] The situation develops. Our correspondent reports \
             from the forward positions."
        );
        telegram_log.entries.push((turn, text));
    }
}

fn stub_telegram_text(turn: u8, e: impl std::fmt::Display) -> String {
    warn!("LLM telegram generation failed for turn {turn}: {e}");
    format!(
        "[Turn {turn}] The situation develops. Our correspondent reports \
         from the forward positions."
    )
}

/// Persist new telegram entries to the game's artifact directory
/// (`games/<game>/telegrams.md`), native only. No-op on wasm.
pub(crate) fn save_telegram_artifacts(
    recorder: Res<crate::game_record::GameRecorder>,
    mut telegram_log: ResMut<TelegramLog>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (&recorder, &mut telegram_log);
    }
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

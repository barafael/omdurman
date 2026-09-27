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
                omdurman_rules::turn_summary::TurnEventRecord::Movement { .. }
                    | omdurman_rules::turn_summary::TurnEventRecord::VpScored { .. }
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
        telegram_log
            .fallbacks
            .insert(turn, fallback_telegram(summary));
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
                        let text = result.unwrap_or_else(|e| {
                            warn!("LLM telegram generation failed for turn {turn}: {e}");
                            telegram_log
                                .fallbacks
                                .get(&turn)
                                .cloned()
                                .unwrap_or_else(|| stub_telegram_text(turn))
                        });
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
        let text = telegram_log
            .fallbacks
            .get(&turn)
            .cloned()
            .unwrap_or_else(|| stub_telegram_text(turn));
        telegram_log.entries.push((turn, text));
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

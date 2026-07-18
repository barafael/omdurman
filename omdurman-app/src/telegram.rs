use bevy::prelude::*;
use bevy::tasks::futures::check_ready;

use crate::llm::{CompletionTag, LlmConfig, PendingCompletions, spawn_completion};

#[derive(Resource, Default)]
pub struct TelegramLog {
    pub entries: Vec<(u8, String)>,
    pub last_processed: usize,
    pub pending_stubs: Vec<u8>,
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

    let stubs: Vec<u8> = telegram_log.pending_stubs.drain(..).collect();
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

use bevy::prelude::*;
use omdurman_rules::turn_summary::TurnSummary;

/// Resource holding the per-turn telegrams generated so far.
#[derive(Resource, Default)]
pub struct TelegramLog {
    pub entries: Vec<(u8, String)>,
    /// Index into `GameState.turn_summaries` of the last processed entry.
    pub last_processed: usize,
}

/// Stub system: watches for new turn summaries and formats a placeholder telegram.
///
/// Once an LLM integration is wired in, this will call the LLM with the
/// structured `TurnSummary` data and store the generated dispatch.
pub(crate) fn generate_telegrams(
    game_state: Option<Res<crate::GameStateResource>>,
    mut telegram_log: ResMut<TelegramLog>,
) {
    let Some(state) = game_state else { return };
    let summaries = &state.0.turn_summaries;
    let len = summaries.len();
    if len <= telegram_log.last_processed {
        return;
    }
    for summary in &summaries[telegram_log.last_processed..] {
        let text = format_stub_telegram(summary);
        telegram_log
            .entries
            .push((summary.turn.value(), text));
    }
    telegram_log.last_processed = len;
}

/// Format a stub telegram from a turn summary.
///
/// This is a placeholder — the real implementation will call an LLM
/// with the structured data and period-appropriate prompt.
fn format_stub_telegram(summary: &TurnSummary) -> String {
    let event_count = summary.events.len();
    let time = summary.time;
    let day_night = summary.day_night;
    format!(
        "[TELEGRAM — Turn {} ({}, {:?})]\n\
         {} events recorded. LLM dispatch generation pending.",
        summary.turn.value(), time, day_night, event_count,
    )
}

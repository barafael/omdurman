//! The end-of-game newspaper. Every peer composes the front page itself
//! from the game state and the filed telegrams
//! ([`omdurman_rules::press::gazette::front_page`]): the generator is
//! deterministic, so all peers -- and a replay -- read the same page.

use bevy::prelude::*;
use omdurman_rules::press::gazette::FrontPage;

/// The composed front page, once the game is over.
#[derive(Resource, Default)]
pub struct NewspaperReport {
    pub page: Option<FrontPage>,
    /// What the page was composed from: (finished turns, filed telegrams).
    composed_from: (usize, usize),
    /// Artifact file written (or nothing to write on wasm).
    pub(crate) saved: bool,
}

/// Compose (or recompose, should a telegram arrive late) the front page of
/// a finished game; clear it when the shown game is not over.
pub(crate) fn compose_newspaper(
    game_state: Option<Res<crate::GameStateResource>>,
    press: Res<crate::telegram::TelegramLog>,
    mut report: ResMut<NewspaperReport>,
) {
    let Some(state) = game_state else { return };
    if !state.0.game_over {
        if report.page.is_some() {
            *report = NewspaperReport::default();
        }
        return;
    }
    let from = (state.0.turn_summaries.len(), press.entries.len());
    if report.page.is_some() && report.composed_from == from {
        return;
    }
    let mut telegrams = press.entries.clone();
    telegrams.sort_by_key(|(turn, _)| *turn);
    report.page = Some(omdurman_rules::press::gazette::front_page(
        &state.0, &telegrams,
    ));
    report.composed_from = from;
    report.saved = false;
}

/// The front page as plain text (the artifact file, and tests). The web
/// build keeps no artifacts (no file system), so it never calls this.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub(crate) fn page_text(page: &FrontPage) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n", page.masthead));
    out.push_str(&format!(
        "{} | {} | {}\n\n",
        page.issue, page.date, page.price
    ));
    out.push_str(&format!("## {}\n\n", page.headline));
    out.push_str(&format!("### {}\n", page.lead.head));
    for deck in &page.lead.decks {
        out.push_str(&format!("*{deck}*\n"));
    }
    out.push('\n');
    for (i, paragraph) in page.lead.paragraphs.iter().enumerate() {
        match (&page.lead.dateline, i) {
            (Some(dateline), 0) => out.push_str(&format!("{dateline} {paragraph}\n\n")),
            _ => out.push_str(&format!("{paragraph}\n\n")),
        }
    }
    if !page.chronicle.is_empty() {
        out.push_str("### THE COURSE OF THE BATTLE\n");
        for (head, text) in &page.chronicle {
            let stop = if head.ends_with('.') { "" } else { "." };
            out.push_str(&format!("*{head}{stop}* {text}\n"));
        }
        out.push('\n');
    }
    out.push_str("### LATE TELEGRAMS\n");
    for (head, text) in &page.telegrams {
        out.push_str(&format!("{head} {text}\n"));
    }
    if !page.roll_of_honour.is_empty() {
        out.push_str(&format!(
            "\n### ROLL OF HONOUR\n{}\n",
            page.roll_of_honour.join(", ")
        ));
    }
    for feature in &page.features {
        out.push_str(&format!(
            "\n### {}\n{}\n",
            feature.head,
            feature.paragraphs.join(" ")
        ));
    }
    out
}

/// Persist the front page to the game's artifact directory
/// (`games/<game>/newspaper.md`), native only. No-op on wasm.
#[cfg_attr(target_arch = "wasm32", allow(unused_variables, unused_mut))]
pub(crate) fn save_newspaper_artifact(
    recorder: Res<crate::game_record::GameRecorder>,
    mut report: ResMut<NewspaperReport>,
    mut retry: Local<crate::game_record::WriteRetry>,
) {
    if report.saved {
        return;
    }
    let Some(page) = report.page.as_ref() else {
        return;
    };
    #[cfg(target_arch = "wasm32")]
    {
        let _ = page;
        report.saved = true;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(dir) = recorder.artifacts_dir() else {
            return;
        };
        let path = format!("{dir}/newspaper.md");
        // Retried with a backoff, warned about once per failing streak (see
        // `WriteRetry`): not rewritten and warned about every frame.
        retry.target(&path);
        if !retry.due() {
            return;
        }
        match std::fs::write(&path, page_text(page)) {
            Ok(()) => {
                if retry.succeeded() {
                    info!(%path, "newspaper artifact writable again");
                }
                report.saved = true;
            }
            // Leave `saved` clear so the write is retried (after the backoff).
            Err(error) => {
                if retry.failed() {
                    warn!(%error, %path, "failed to write newspaper artifact; will retry");
                } else {
                    debug!(%error, %path, "newspaper artifact still not writable");
                }
            }
        }
    }
}

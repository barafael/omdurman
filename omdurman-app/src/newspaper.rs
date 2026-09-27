use bevy::prelude::*;
use bevy::tasks::futures::check_ready;

use crate::llm::{CompletionTag, LlmConfig, PendingCompletions, spawn_completion};

#[derive(Resource, Default)]
pub struct NewspaperReport {
    pub masthead: String,
    pub date_line: String,
    pub headline: String,
    pub subhead: String,
    pub scenario: String,
    pub turns_played: u8,
    pub result_key: String,
    pub paragraphs: Vec<String>,
}

#[derive(Resource, Default)]
pub struct NewspaperLlmState {
    pub dispatched: bool,
    pub completed: bool,
    /// Artifact file already written (or nothing to write on wasm).
    pub saved: bool,
}

/// At game over, every peer sets up the Gazette's fixed parts (masthead,
/// headline, ...: chosen from the result, so identical everywhere); only the
/// host writes the report paragraphs -- its flavour model, or the template's
/// prompts when there is none -- and publishes them (see
/// [`crate::telegram::PressDesk`]), so everyone reads the same report.
pub(crate) fn generate_newspaper(
    game_state: Option<Res<crate::GameStateResource>>,
    llm_config: Res<LlmConfig>,
    mut report: ResMut<NewspaperReport>,
    mut llm_state: ResMut<NewspaperLlmState>,
    mut pending: ResMut<PendingCompletions>,
    mut desk: crate::telegram::PressDesk,
    mut press: ResMut<crate::telegram::TelegramLog>,
) {
    let Some(state) = game_state else { return };
    if !state.0.game_over {
        return;
    }
    let Some(result) = state.0.game_result else {
        return;
    };

    let template = omdurman_rules::newspaper::newspaper_template(
        result,
        state.0.gordon_eliminated_turn.is_some(),
    );

    if report.headline.is_empty() {
        report.masthead = "THE LONDON GAZETTE".to_string();
        report.date_line = result.date_line().to_string();
        report.headline = template.headline.to_string();
        report.subhead = template.subhead.to_string();
        report.scenario = state.0.scenario.label().to_string();
        report.turns_played = state.0.current_turn.value();
        report.result_key = result.display_key();
    }

    // Written once per game: not by a guest, and not again by a peer that
    // became host after the old host's Gazette was filed.
    if llm_state.dispatched || press.gazette.is_some() || !desk.writes() {
        return;
    }
    llm_state.dispatched = true;
    if llm_config.has_key() {
        let mut facts = Vec::new();
        if state.0.scenario == omdurman_types::Scenario::FallOfKhartoum {
            facts.push(match state.0.gordon_eliminated_turn {
                Some(turn) => format!(
                    "General Gordon was killed when the Mahdi's forces reached the palace on turn {}.",
                    turn.value()
                ),
                None => "General Gordon survived; the palace never fell.".to_string(),
            });
        }
        let prompt = omdurman_rules::newspaper::build_newspaper_prompt(
            template,
            &state.0.turn_summaries,
            result,
            &facts,
        );
        spawn_completion(
            &llm_config,
            "",
            &prompt,
            CompletionTag::Newspaper,
            crate::llm::NEWSPAPER_MAX_TOKENS,
            &mut pending,
        );
    } else {
        let paragraphs = template
            .highlight_prompts
            .iter()
            .map(|hint| format!("[Stub] {hint}"))
            .collect();
        desk.publish(&mut press, omdurman_net::GameEvent::Gazette { paragraphs });
    }
}

/// Host: publish the report the flavour model returns (or a short apology
/// if it failed) as the Gazette's paragraphs.
pub(crate) fn poll_newspaper_completion(
    mut pending: ResMut<PendingCompletions>,
    mut desk: crate::telegram::PressDesk,
    mut press: ResMut<crate::telegram::TelegramLog>,
) {
    let Some(i) = pending
        .items
        .iter()
        .position(|item| matches!(item.tag, CompletionTag::Newspaper))
    else {
        return;
    };
    let Some(result) = check_ready(&mut pending.items[i].task) else {
        return;
    };
    pending.items.swap_remove(i);
    let paragraphs = match result {
        Ok(text) => text
            .split("\n\n")
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect(),
        Err(e) => {
            warn!("LLM newspaper generation failed: {e}");
            vec!["Our correspondent was unable to file a full report.".to_string()]
        }
    };
    desk.publish(&mut press, omdurman_net::GameEvent::Gazette { paragraphs });
}

/// Every peer: show the Gazette paragraphs once they are filed.
pub(crate) fn adopt_filed_gazette(
    press: Res<crate::telegram::TelegramLog>,
    mut report: ResMut<NewspaperReport>,
    mut llm_state: ResMut<NewspaperLlmState>,
) {
    if llm_state.completed {
        return;
    }
    if let Some(paragraphs) = &press.gazette {
        report.paragraphs = paragraphs.clone();
        llm_state.completed = true;
    }
}

/// Persist the finished newspaper report to the game's artifact directory
/// (`games/<game>/newspaper.md`) once generation completed, native only.
/// No-op on wasm.
#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
pub(crate) fn save_newspaper_artifact(
    recorder: Res<crate::game_record::GameRecorder>,
    report: Res<NewspaperReport>,
    mut llm_state: ResMut<NewspaperLlmState>,
) {
    if !llm_state.completed || llm_state.saved {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    {
        llm_state.saved = true;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(dir) = recorder.artifacts_dir() else {
            return;
        };
        let path = format!("{dir}/newspaper.md");
        let result = (|| -> std::io::Result<()> {
            use std::io::Write;
            let mut f = std::fs::File::create(&path)?;
            writeln!(f, "# {}", report.masthead)?;
            writeln!(f, "{}", report.date_line)?;
            writeln!(f)?;
            writeln!(f, "## {}", report.headline)?;
            if !report.subhead.is_empty() {
                writeln!(f, "*{}*", report.subhead)?;
            }
            writeln!(f)?;
            for paragraph in &report.paragraphs {
                writeln!(f, "{paragraph}")?;
                writeln!(f)?;
            }
            writeln!(
                f,
                "---\nScenario: {} | Turns played: {} | Result: {}",
                report.scenario, report.turns_played, report.result_key
            )?;
            Ok(())
        })();
        match result {
            Ok(()) => llm_state.saved = true,
            // Leave `saved` clear so the write is retried next frame.
            Err(error) => warn!(%error, %path, "failed to write newspaper artifact"),
        }
    }
}

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

pub(crate) fn generate_newspaper(
    game_state: Option<Res<crate::GameStateResource>>,
    llm_config: Res<LlmConfig>,
    mut report: ResMut<NewspaperReport>,
    mut llm_state: ResMut<NewspaperLlmState>,
    mut pending: ResMut<PendingCompletions>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(state) = game_state else { return };
    if !state.0.game_over {
        return;
    }
    let Some(result) = state.0.game_result else {
        return;
    };

    let template = omdurman_rules::newspaper::newspaper_template(result);

    report.masthead = "THE LONDON GAZETTE".to_string();
    report.date_line = "September 1898".to_string();
    report.headline = template.headline.to_string();
    report.subhead = template.subhead.to_string();
    report.scenario = format!("{:?}", state.0.scenario);
    report.turns_played = state.0.current_turn.value();
    report.result_key = result.display_key();

    if !llm_state.dispatched {
        if llm_config.has_key() {
            let prompt = omdurman_rules::newspaper::build_newspaper_prompt(
                template,
                &state.0.turn_summaries,
                result,
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
            report.paragraphs = template
                .highlight_prompts
                .iter()
                .map(|hint| format!("[Stub] {hint}"))
                .collect();
            llm_state.completed = true;
        }
        llm_state.dispatched = true;
    }

    *done = true;
}

pub(crate) fn poll_newspaper_completion(
    mut pending: ResMut<PendingCompletions>,
    mut report: ResMut<NewspaperReport>,
    mut llm_state: ResMut<NewspaperLlmState>,
) {
    if llm_state.completed || !llm_state.dispatched {
        return;
    }

    let mut i = 0;
    while i < pending.items.len() {
        if matches!(pending.items[i].tag, CompletionTag::Newspaper) {
            if let Some(result) = check_ready(&mut pending.items[i].task) {
                let _item = pending.items.swap_remove(i);
                match result {
                    Ok(text) => {
                        report.paragraphs = text
                            .split("\n\n")
                            .filter(|s| !s.trim().is_empty())
                            .map(|s| s.trim().to_string())
                            .collect();
                    }
                    Err(e) => {
                        warn!("LLM newspaper generation failed: {e}");
                        report.paragraphs =
                            vec!["Our correspondent was unable to file a full report.".to_string()];
                    }
                }
                llm_state.completed = true;
                return;
            }
            break;
        }
        i += 1;
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

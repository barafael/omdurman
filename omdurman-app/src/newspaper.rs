use bevy::prelude::*;

/// Resource holding the generated newspaper report for the current game.
///
/// Populated by [`generate_newspaper`] once when the game ends. The UI reads
/// from this to render the in-app newspaper display.
#[derive(Resource, Default)]
pub struct NewspaperReport {
    /// The newspaper masthead line (e.g. "THE LONDON GAZETTE").
    pub masthead: String,
    /// Date line (e.g. "September 1898").
    pub date_line: String,
    /// The headline from the template.
    pub headline: String,
    /// The subhead from the template.
    pub subhead: String,
    /// Scenario name for the stats display.
    pub scenario: String,
    /// Final turn number.
    pub turns_played: u8,
    /// The result key (e.g. "historical_dervish_10").
    pub result_key: String,
}

/// System: when the game ends, look up the newspaper template and populate
/// the report resource with template data + game stats.
pub(crate) fn generate_newspaper(
    game_state: Option<Res<crate::GameStateResource>>,
    mut report: ResMut<NewspaperReport>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(state) = game_state else { return };
    if !state.0.game_over {
        return;
    }
    let Some(result) = state.0.game_result else { return };

    let template = omdurman_rules::newspaper::newspaper_template(result);

    report.masthead = "THE LONDON GAZETTE".to_string();
    report.date_line = "September 1898".to_string();
    report.headline = template.headline.to_string();
    report.subhead = template.subhead.to_string();
    report.scenario = format!("{:?}", state.0.scenario);
    report.turns_played = state.0.current_turn.value();
    report.result_key = result.display_key();

    *done = true;
}

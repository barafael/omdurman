use crate::{GameResult, HistoricalVictoryLevel};
use crate::turn_summary::TurnSummary;

/// A pre-generated newspaper template for a specific game outcome.
pub struct NewspaperTemplate {
    pub headline: &'static str,
    pub subhead: &'static str,
    pub highlight_prompts: &'static [&'static str],
}

/// Look up the newspaper template for a given typed game result.
///
/// `result` is the [`GameResult`] stored on `GameState::game_result` by
/// `finish_game` (rulebook §9.14, §9.24, §9.35).
pub fn newspaper_template(result: GameResult) -> &'static NewspaperTemplate {
    match result {
        GameResult::Campaign(level) => campaign_template(level),
        GameResult::Historical { ae, d } => historical_template(ae, d),
        GameResult::FoK(level) => fok_template(level),
    }
}

fn campaign_template(level: crate::CampaignVictoryLevel) -> &'static NewspaperTemplate {
    use crate::CampaignVictoryLevel as L;
    use crate::Player as P;
    match level {
        L::Decisive(P::AngloEgyptian) => &NEWSPAPER_CAMPAIGN_DECISIVE_AE,
        L::Tactical(P::AngloEgyptian) => &NEWSPAPER_CAMPAIGN_TACTICAL_AE,
        L::Marginal(P::AngloEgyptian) => &NEWSPAPER_CAMPAIGN_MARGINAL_AE,
        L::Draw => &NEWSPAPER_CAMPAIGN_DRAW,
        L::Marginal(P::Dervish) => &NEWSPAPER_CAMPAIGN_MARGINAL_D,
        L::Tactical(P::Dervish) => &NEWSPAPER_CAMPAIGN_TACTICAL_D,
        L::Decisive(P::Dervish) => &NEWSPAPER_CAMPAIGN_DECISIVE_D,
    }
}

/// Pick a Historical template from the two per-side levels (§9.24). The net
/// result is `ae_level - d_level` (each cast to its discriminant); positive
/// favours the Anglo-Egyptian, negative the Dervish, and zero is a draw. The
/// magnitude selects which rung of the winner's ladder applies.
fn historical_template(
    ae: HistoricalVictoryLevel,
    d: HistoricalVictoryLevel,
) -> &'static NewspaperTemplate {
    let net = ae as i16 - d as i16;
    if net > 0 {
        // Anglo-Egyptian victory; the level is the AE rung.
        match ae {
            HistoricalVictoryLevel::Decisive => &NEWSPAPER_HISTORICAL_AE_DECISIVE,
            HistoricalVictoryLevel::Strategic => &NEWSPAPER_HISTORICAL_AE_STRATEGIC,
            HistoricalVictoryLevel::Tactical => &NEWSPAPER_HISTORICAL_AE_TACTICAL,
            HistoricalVictoryLevel::Marginal => &NEWSPAPER_HISTORICAL_AE_MARGINAL,
            HistoricalVictoryLevel::Draw => &NEWSPAPER_HISTORICAL_DRAW,
        }
    } else if net < 0 {
        // Dervish victory; the level is the D rung.
        match d {
            HistoricalVictoryLevel::Decisive => &NEWSPAPER_HISTORICAL_D_DECISIVE,
            HistoricalVictoryLevel::Strategic => &NEWSPAPER_HISTORICAL_D_STRATEGIC,
            HistoricalVictoryLevel::Tactical => &NEWSPAPER_HISTORICAL_D_TACTICAL,
            HistoricalVictoryLevel::Marginal => &NEWSPAPER_HISTORICAL_D_MARGINAL,
            HistoricalVictoryLevel::Draw => &NEWSPAPER_HISTORICAL_DRAW,
        }
    } else {
        &NEWSPAPER_HISTORICAL_DRAW
    }
}

fn fok_template(level: crate::FoKVictoryLevel) -> &'static NewspaperTemplate {
    use crate::FoKVictoryLevel as L;
    match level {
        L::BritishDecisive => &NEWSPAPER_FOK_BRITISH_DECISIVE,
        L::BritishTactical => &NEWSPAPER_FOK_BRITISH_TACTICAL,
        L::BritishMarginal => &NEWSPAPER_FOK_BRITISH_MARGINAL,
        L::DervishMarginal => &NEWSPAPER_FOK_DERVISH_MARGINAL,
        L::DervishTactical => &NEWSPAPER_FOK_DERVISH_TACTICAL,
        L::DervishDecisive => &NEWSPAPER_FOK_DERVISH_DECISIVE,
    }
}

/// Build a prompt string for the LLM to generate newspaper paragraphs.
pub fn build_newspaper_prompt(
    template: &NewspaperTemplate,
    summaries: &[TurnSummary],
    result: GameResult,
) -> String {
    let result_key = result.display_key();
    let total_turns = summaries.len();
    let mut prompt = format!(
        "You are a correspondent for The Times of London, September 1898.\n\
         Write a brief newspaper report (2-4 short paragraphs) about the \
         Battle of Omdurman.\n\n\
         HEADLINE: {}\n\
         SUBHEAD: {}\n\n\
         Result: {}\n\
         Total turns played: {}\n\n",
        template.headline, template.subhead, result_key, total_turns,
    );

    prompt.push_str("Write about these aspects:\n");
    for (i, hint) in template.highlight_prompts.iter().enumerate() {
        prompt.push_str(&format!("{}. {}\n", i + 1, hint));
    }

    prompt.push_str(
        "\nKeep to 2-4 brief paragraphs. Victorian newspaper tone. \
         Do not repeat the headline or subhead in the body.",
    );

    prompt
}

// ---------------------------------------------------------------------------
// Campaign templates (7 outcomes)
// ---------------------------------------------------------------------------

static NEWSPAPER_CAMPAIGN_DECISIVE_AE: NewspaperTemplate = NewspaperTemplate {
    headline: "GLORIOUS VICTORY \u{2014} THE DERVISH HOST SHATTERED",
    subhead: "Kitchener\u{2019}s Forces Carry the Day at Omdurman",
    highlight_prompts: &[
        "Describe the Anglo-Egyptian advance and the storming of the zariba",
        "Note the heavy casualties inflicted on the Dervish forces",
        "Mention the fate of the Khalifa or the fall of the Mahdi\u{2019}s Tomb if applicable",
        "Comment on the strategic significance of the victory for the Sudan campaign",
    ],
};

static NEWSPAPER_CAMPAIGN_TACTICAL_AE: NewspaperTemplate = NewspaperTemplate {
    headline: "BLOODY BUT VICTORIOUS \u{2014} KITCHENER\u{2019}S FORCES TRIUMPH",
    subhead: "Tactical Victory for the Anglo-Egyptian Army",
    highlight_prompts: &[
        "Describe the hard-fought engagement and the Anglo-Egyptian casualties",
        "Note the key actions that tipped the balance",
        "Mention the Dervish resistance and their losses",
    ],
};

static NEWSPAPER_CAMPAIGN_MARGINAL_AE: NewspaperTemplate = NewspaperTemplate {
    headline: "NARROW SUCCESS ON THE NILE",
    subhead: "Anglo-Egyptian Forces Edge Out the Dervish",
    highlight_prompts: &[
        "Describe the closely contested battle",
        "Note the slim margin of victory",
        "Comment on the cost of the engagement",
    ],
};

static NEWSPAPER_CAMPAIGN_DRAW: NewspaperTemplate = NewspaperTemplate {
    headline: "STALEMATE AT OMDURMAN",
    subhead: "Heavy Casualties on Both Sides; No Decisive Result",
    highlight_prompts: &[
        "Describe the fierce fighting with no clear victor",
        "Note the casualties on both sides",
        "Comment on the implications for the campaign",
    ],
};

static NEWSPAPER_CAMPAIGN_MARGINAL_D: NewspaperTemplate = NewspaperTemplate {
    headline: "DERVISH FORCES HOLD THEIR GROUND",
    subhead: "Anglo-Egyptian Advance Checked at Omdurman",
    highlight_prompts: &[
        "Describe the Dervish defence and their successful resistance",
        "Note the Anglo-Egyptian setbacks",
        "Comment on the state of the campaign",
    ],
};

static NEWSPAPER_CAMPAIGN_TACTICAL_D: NewspaperTemplate = NewspaperTemplate {
    headline: "REVERSE FOR KITCHENER \u{2014} DERVISH TRIUMPH",
    subhead: "The Mahdi\u{2019}s Warriors Repel the Anglo-Egyptian Forces",
    highlight_prompts: &[
        "Describe the Dervish victory and their tactics",
        "Note the Anglo-Egyptian losses and retreat",
        "Comment on the political fallout in London",
    ],
};

static NEWSPAPER_CAMPAIGN_DECISIVE_D: NewspaperTemplate = NewspaperTemplate {
    headline: "CATASTROPHE ON THE NILE",
    subhead: "The Anglo-Egyptian Army Routed; General Gordon\u{2019}s Worst Fears Realised",
    highlight_prompts: &[
        "Describe the destruction of the Anglo-Egyptian force",
        "Note the scale of the defeat and its causes",
        "Comment on the implications for British prestige in Egypt",
    ],
};

// ---------------------------------------------------------------------------
// Historical templates (9 net results, simplified to key outcomes)
// ---------------------------------------------------------------------------

static NEWSPAPER_HISTORICAL_AE_DECISIVE: NewspaperTemplate = NewspaperTemplate {
    headline: "COMPLETE VICTORY \u{2014} THE DERVISH POWER BROKEN",
    subhead: "A Decisive Triumph for Anglo-Egyptian Arms",
    highlight_prompts: &[
        "Describe the overwhelming Anglo-Egyptian success",
        "Note the total destruction of the Dervish forces",
        "Comment on the restoration of order in the Sudan",
    ],
};

static NEWSPAPER_HISTORICAL_AE_STRATEGIC: NewspaperTemplate = NewspaperTemplate {
    headline: "STRATEGIC VICTORY FOR THE ANGLO-EGYPTIAN FORCES",
    subhead: "Dervish Resistance Largely Crushed",
    highlight_prompts: &[
        "Describe the effective destruction of Dervish military capacity",
        "Note the key engagements that secured the victory",
    ],
};

static NEWSPAPER_HISTORICAL_AE_TACTICAL: NewspaperTemplate = NewspaperTemplate {
    headline: "TACTICAL VICTORY AT OMDURMAN",
    subhead: "Anglo-Egyptian Forces Prevail in Hard-Fought Engagement",
    highlight_prompts: &[
        "Describe the battle and its tactical course",
        "Note the Dervish losses that secured the result",
    ],
};

static NEWSPAPER_HISTORICAL_AE_MARGINAL: NewspaperTemplate = NewspaperTemplate {
    headline: "SLIGHT ADVANTAGE TO THE ANGLO-EGYPTIAN FORCES",
    subhead: "A Marginal Result After Fierce Fighting",
    highlight_prompts: &[
        "Describe the closely contested engagement",
        "Note the limited gains on both sides",
    ],
};

static NEWSPAPER_HISTORICAL_DRAW: NewspaperTemplate = NewspaperTemplate {
    headline: "INDECISIVE ENGAGEMENT AT OMDURMAN",
    subhead: "Neither Side Claims a Clear Victory",
    highlight_prompts: &[
        "Describe the inconclusive fighting",
        "Note the casualties on both sides",
    ],
};

static NEWSPAPER_HISTORICAL_D_MARGINAL: NewspaperTemplate = NewspaperTemplate {
    headline: "DERVISH FORCES REPulse THE ANGLO-EGYPTIAN ATTACK",
    subhead: "A Marginal Success for the Defenders",
    highlight_prompts: &[
        "Describe the Dervish defensive success",
        "Note the Anglo-Egyptian failure to achieve objectives",
    ],
};

static NEWSPAPER_HISTORICAL_D_TACTICAL: NewspaperTemplate = NewspaperTemplate {
    headline: "DERVISH TACTICAL VICTORY \u{2014} ANGLO-EGYPTIAN SETBACK",
    subhead: "The Defenders Inflict a Sharp Rebuke",
    highlight_prompts: &[
        "Describe the Dervish victory and its tactical significance",
        "Note the Anglo-Egyptian losses",
    ],
};

static NEWSPAPER_HISTORICAL_D_STRATEGIC: NewspaperTemplate = NewspaperTemplate {
    headline: "DERVISH STRATEGIC VICTORY \u{2014} THE ADVANCE HALTED",
    subhead: "Anglo-Egyptian Forces Suffer a Serious Reverse",
    highlight_prompts: &[
        "Describe the scale of the Dervish victory",
        "Note the Anglo-Egyptian withdrawal",
        "Comment on the political consequences",
    ],
};

static NEWSPAPER_HISTORICAL_D_DECISIVE: NewspaperTemplate = NewspaperTemplate {
    headline: "CATASTROPHIC DEFEAT FOR THE ANGLO-EGYPTIAN EXPEDITION",
    subhead: "The Dervish Host Annihilates the Invading Force",
    highlight_prompts: &[
        "Describe the total destruction of the Anglo-Egyptian force",
        "Note the scale of the disaster",
        "Comment on the shock to the British public",
    ],
};

// ---------------------------------------------------------------------------
// Fall of Khartoum templates (6 outcomes)
// ---------------------------------------------------------------------------

static NEWSPAPER_FOK_BRITISH_DECISIVE: NewspaperTemplate = NewspaperTemplate {
    headline: "RELIEF OF KHARTOUM \u{2014} GORDON SAVED, THE MAHDI\u{2019}S POWER BROKEN",
    subhead: "A Decisive British Triumph in the Sudan",
    highlight_prompts: &[
        "Describe the relief of Khartoum and the fate of General Gordon",
        "Note the destruction of the Dervish forces",
        "Comment on the restoration of British prestige",
    ],
};

static NEWSPAPER_FOK_BRITISH_TACTICAL: NewspaperTemplate = NewspaperTemplate {
    headline: "KHARTOUM RELIEVED \u{2014} GORDON SAVED",
    subhead: "Tactical Victory for the Relief Force",
    highlight_prompts: &[
        "Describe the relief of Khartoum",
        "Note the cost of the operation",
        "Comment on Gordon\u{2019}s condition",
    ],
};

static NEWSPAPER_FOK_BRITISH_MARGINAL: NewspaperTemplate = NewspaperTemplate {
    headline: "KHARTOUM REACHED \u{2014} GORDON SAFE",
    subhead: "A Narrow but Welcome Success",
    highlight_prompts: &[
        "Describe the arrival at Khartoum",
        "Note the limited nature of the victory",
    ],
};

static NEWSPAPER_FOK_DERVISH_MARGINAL: NewspaperTemplate = NewspaperTemplate {
    headline: "THE SIEGE OF KHARTOUM CONTINUES",
    subhead: "British Relief Attempt Checked",
    highlight_prompts: &[
        "Describe the failed relief attempt",
        "Note Gordon\u{2019}s continued peril",
    ],
};

static NEWSPAPER_FOK_DERVISH_TACTICAL: NewspaperTemplate = NewspaperTemplate {
    headline: "RELIEF FORCE REPULSED \u{2014} GORDON IN GRAVE DANGER",
    subhead: "Dervish Forces Prevail at Khartoum",
    highlight_prompts: &[
        "Describe the defeat of the relief force",
        "Note the implications for Gordon\u{2019}s survival",
        "Comment on the political crisis in London",
    ],
};

static NEWSPAPER_FOK_DERVISH_DECISIVE: NewspaperTemplate = NewspaperTemplate {
    headline: "FALL OF KHARTOUM \u{2014} GORDON LOST",
    subhead: "British Humiliation; The Mahdi\u{2019}s Power Unchallenged",
    highlight_prompts: &[
        "Describe the fall of Khartoum and Gordon\u{2019}s death",
        "Note the destruction of the British garrison",
        "Comment on the national mourning and political fallout",
    ],
};

// ---------------------------------------------------------------------------
// Helpers for formatting turn summaries for the LLM
// ---------------------------------------------------------------------------

/// Format all turn summaries as a block of text for LLM input.
pub fn format_summaries_for_llm(summaries: &[TurnSummary]) -> String {
    summaries
        .iter()
        .map(|s| s.format_for_llm())
        .collect::<Vec<_>>()
        .join("\n")
}

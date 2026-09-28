use crate::turn_summary::TurnSummary;
use crate::{GameResult, HistoricalVictoryLevel};

/// A pre-generated newspaper template for a specific game outcome.
pub struct NewspaperTemplate {
    pub headline: &'static str,
    pub subhead: &'static str,
    pub highlight_prompts: &'static [&'static str],
}

/// Look up the newspaper template for a given typed game result.
///
/// `result` is the [`GameResult`] stored on `GameState::game_result` by
/// `finish_game` (rulebook §9.14, §9.24, §9.35). `gordon_fell` is whether
/// GORDON was eliminated; only FALL OF KHARTOUM consults it, because there
/// the §9.35 loss penalty can turn his death into a British result -- a
/// headline chosen by level alone would announce him saved.
pub fn newspaper_template(result: GameResult, gordon_fell: bool) -> &'static NewspaperTemplate {
    match result {
        GameResult::Campaign(level) => campaign_template(level),
        GameResult::Historical { ae, d } => historical_template(ae, d),
        GameResult::FoK(level) => fok_template(level, gordon_fell),
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

/// Pick a Historical template from the two per-side levels (§9.24): the net
/// result -- the higher level less the lower, read on the same scale -- goes
/// to the side with the higher level; equal or adjacent levels are a draw.
/// (Anglo-Egyptian Strategic against a Dervish Draw nets a Tactical victory,
/// not a Strategic one.)
fn historical_template(
    ae: HistoricalVictoryLevel,
    d: HistoricalVictoryLevel,
) -> &'static NewspaperTemplate {
    use crate::Player as P;
    use HistoricalVictoryLevel as L;
    match HistoricalVictoryLevel::net(ae, d) {
        (None, _) | (_, L::Draw) => &NEWSPAPER_HISTORICAL_DRAW,
        (Some(P::AngloEgyptian), level) => match level {
            L::Decisive => &NEWSPAPER_HISTORICAL_AE_DECISIVE,
            L::Strategic => &NEWSPAPER_HISTORICAL_AE_STRATEGIC,
            L::Tactical => &NEWSPAPER_HISTORICAL_AE_TACTICAL,
            L::Marginal | L::Draw => &NEWSPAPER_HISTORICAL_AE_MARGINAL,
        },
        (Some(P::Dervish), level) => match level {
            L::Decisive => &NEWSPAPER_HISTORICAL_D_DECISIVE,
            L::Strategic => &NEWSPAPER_HISTORICAL_D_STRATEGIC,
            L::Tactical => &NEWSPAPER_HISTORICAL_D_TACTICAL,
            L::Marginal | L::Draw => &NEWSPAPER_HISTORICAL_D_MARGINAL,
        },
    }
}

fn fok_template(level: crate::FoKVictoryLevel, gordon_fell: bool) -> &'static NewspaperTemplate {
    use crate::FoKVictoryLevel as L;
    match (level, gordon_fell) {
        (L::BritishDecisive, false) => &NEWSPAPER_FOK_BRITISH_DECISIVE,
        (L::BritishTactical, false) => &NEWSPAPER_FOK_BRITISH_TACTICAL,
        (L::BritishMarginal, false) => &NEWSPAPER_FOK_BRITISH_MARGINAL,
        // §9.35: GORDON fell, but the Dervish losses cost the victory.
        (L::BritishDecisive, true) => &NEWSPAPER_FOK_GORDON_FELL_BRITISH_DECISIVE,
        (L::BritishTactical, true) => &NEWSPAPER_FOK_GORDON_FELL_BRITISH_TACTICAL,
        (L::BritishMarginal, true) => &NEWSPAPER_FOK_GORDON_FELL_BRITISH_MARGINAL,
        // A Dervish level needs GORDON's death (§9.35).
        (L::DervishMarginal, _) => &NEWSPAPER_FOK_DERVISH_MARGINAL,
        (L::DervishTactical, _) => &NEWSPAPER_FOK_DERVISH_TACTICAL,
        (L::DervishDecisive, _) => &NEWSPAPER_FOK_DERVISH_DECISIVE,
    }
}

/// Build a prompt string for the LLM to generate newspaper paragraphs.
pub fn build_newspaper_prompt(
    template: &NewspaperTemplate,
    summaries: &[TurnSummary],
    result: GameResult,
    facts: &[String],
) -> String {
    let result_key = result.display_key();
    let total_turns = summaries.len();
    let mut prompt = format!(
        "You are a correspondent for The Times of London, {}.\n\
         Write a brief newspaper report (2-4 short paragraphs, at most 250 \
         words total) about {}.\n\n\
         HEADLINE: {}\n\
         SUBHEAD: {}\n\n\
         Result: {}\n\
         Total turns played: {}\n\n",
        result.date_line(),
        result.battle_name(),
        template.headline,
        template.subhead,
        result_key,
        total_turns,
    );

    // Hard facts the report must not contradict (e.g. whether GORDON lived:
    // after §9.35 loss penalties a British result can follow his death).
    if !facts.is_empty() {
        prompt.push_str("Established facts (never contradict these):\n");
        for fact in facts {
            prompt.push_str(&format!("- {fact}\n"));
        }
        prompt.push('\n');
    }
    prompt.push_str("Write about these aspects:\n");
    for (i, hint) in template.highlight_prompts.iter().enumerate() {
        prompt.push_str(&format!("{}. {}\n", i + 1, hint));
    }

    prompt.push_str(
        "\nKeep to 2-4 brief paragraphs, at most 250 words total — stay well \
         under the limit and finish with a closing sentence, never cut off \
         mid-thought. Victorian newspaper tone. \
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

// The scenario is the siege itself (§9.3): the garrison holds Khartoum and
// no relief column is on the map, so the British headlines are about the
// defence, not a relief.

static NEWSPAPER_FOK_BRITISH_DECISIVE: NewspaperTemplate = NewspaperTemplate {
    headline: "KHARTOUM HOLDS \u{2014} GORDON DEFIES THE MAHDI",
    subhead: "The Garrison Throws Back Every Assault",
    highlight_prompts: &[
        "Describe the defence of Khartoum and General Gordon at the palace",
        "Note the Dervish assaults broken on the walls",
        "Comment on the heavy losses of the besiegers",
    ],
};

static NEWSPAPER_FOK_BRITISH_TACTICAL: NewspaperTemplate = NewspaperTemplate {
    headline: "KHARTOUM STANDS \u{2014} GORDON STILL HOLDS THE PALACE",
    subhead: "The Mahdi\u{2019}s Assault Falters",
    highlight_prompts: &[
        "Describe the fighting along the walls",
        "Note the cost of the defence",
        "Comment on Gordon\u{2019}s resolve",
    ],
};

static NEWSPAPER_FOK_BRITISH_MARGINAL: NewspaperTemplate = NewspaperTemplate {
    headline: "KHARTOUM HOLDS ON \u{2014} GORDON SAFE FOR NOW",
    subhead: "A Narrow Reprieve for the Garrison",
    highlight_prompts: &[
        "Describe how close the city came to falling",
        "Note the limited nature of the reprieve",
    ],
};

static NEWSPAPER_FOK_GORDON_FELL_BRITISH_DECISIVE: NewspaperTemplate = NewspaperTemplate {
    headline: "GORDON DIES AT HIS POST \u{2014} THE MAHDI\u{2019}S HOST BLED WHITE",
    subhead: "The Palace Falls, but the Besieging Army Is Shattered",
    highlight_prompts: &[
        "Describe the storming of the palace and Gordon\u{2019}s death",
        "Note the ruinous Dervish losses before the walls",
        "Comment on a victory the Mahdi cannot afford to repeat",
    ],
};

static NEWSPAPER_FOK_GORDON_FELL_BRITISH_TACTICAL: NewspaperTemplate = NewspaperTemplate {
    headline: "GORDON FALLS \u{2014} AT RUINOUS COST TO THE MAHDI",
    subhead: "Khartoum Taken Over the Bodies of Thousands",
    highlight_prompts: &[
        "Describe the fall of the palace and Gordon\u{2019}s death",
        "Note the heavy Dervish casualties",
    ],
};

static NEWSPAPER_FOK_GORDON_FELL_BRITISH_MARGINAL: NewspaperTemplate = NewspaperTemplate {
    headline: "GORDON FALLS \u{2014} THE MAHDI\u{2019}S VICTORY DEARLY BOUGHT",
    subhead: "Khartoum Taken, the Besiegers Much Reduced",
    highlight_prompts: &[
        "Describe the final assault on the palace and Gordon\u{2019}s death",
        "Note what the siege cost the Dervish army",
    ],
};

static NEWSPAPER_FOK_DERVISH_MARGINAL: NewspaperTemplate = NewspaperTemplate {
    headline: "KHARTOUM FALLS AFTER A LONG DEFENCE \u{2014} GORDON KILLED",
    subhead: "The Garrison Overwhelmed at Last",
    highlight_prompts: &[
        "Describe the stubborn defence and the final assault",
        "Note Gordon\u{2019}s death at the palace",
    ],
};

static NEWSPAPER_FOK_DERVISH_TACTICAL: NewspaperTemplate = NewspaperTemplate {
    headline: "KHARTOUM STORMED \u{2014} GORDON KILLED AT THE PALACE",
    subhead: "Dervish Forces Prevail at Khartoum",
    highlight_prompts: &[
        "Describe the storming of the city",
        "Note Gordon\u{2019}s death",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FoKVictoryLevel as L;

    // §9.35: the loss penalty can turn GORDON's death into a British result,
    // so the FoK headline follows his fate, never the level alone.
    #[test]
    fn fok_headlines_never_contradict_gordons_fate() {
        let british = [L::BritishDecisive, L::BritishTactical, L::BritishMarginal];
        for level in british {
            let fell = newspaper_template(GameResult::FoK(level), true).headline;
            assert!(fell.contains("GORDON") && !fell.contains("SAFE"), "{fell}");
            assert!(!fell.contains("SAVED") && !fell.contains("HOLDS"), "{fell}");
            let lived = newspaper_template(GameResult::FoK(level), false).headline;
            assert!(
                !lived.contains("FALL") && !lived.contains("KILLED"),
                "{lived}"
            );
        }
        for level in [L::DervishMarginal, L::DervishTactical, L::DervishDecisive] {
            let headline = newspaper_template(GameResult::FoK(level), true).headline;
            assert!(
                headline.contains("KILLED") || headline.contains("LOST"),
                "{headline}"
            );
        }
    }

    // §9.24: the headline announces the *net* result. Anglo-Egyptian
    // Strategic against a Dervish Draw nets a Tactical victory; the
    // rulebook's worked example (Decisive against Strategic) nets a draw.
    #[test]
    fn historical_headline_follows_the_net_result() {
        use HistoricalVictoryLevel as H;
        let headline = |ae, d| newspaper_template(GameResult::Historical { ae, d }, false).headline;
        assert_eq!(
            headline(H::Strategic, H::Draw),
            NEWSPAPER_HISTORICAL_AE_TACTICAL.headline
        );
        assert_eq!(
            headline(H::Decisive, H::Strategic),
            NEWSPAPER_HISTORICAL_DRAW.headline
        );
        assert_eq!(
            headline(H::Draw, H::Tactical),
            NEWSPAPER_HISTORICAL_D_MARGINAL.headline
        );
        assert_eq!(
            headline(H::Decisive, H::Draw),
            NEWSPAPER_HISTORICAL_AE_STRATEGIC.headline
        );
    }
}

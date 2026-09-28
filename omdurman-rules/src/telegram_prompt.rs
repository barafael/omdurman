use crate::turn_summary::TurnSummary;
use omdurman_types::Scenario;

/// Build a system + user prompt pair for the LLM to generate a military
/// telegram from the Anglo-Egyptian command perspective, set in the
/// scenario's own battle (FALL OF KHARTOUM is Gordon's Khartoum, January
/// 1885 -- not Kitchener's Omdurman).
///
/// Returns `(system_prompt, user_prompt)`.
pub fn build_telegram_prompt(summary: &TurnSummary, scenario: Scenario) -> (String, String) {
    let setting = match scenario {
        Scenario::FallOfKhartoum => {
            "in the Governor-General's palace at Khartoum, January 1885, \
             under siege by the Mahdi's army (General Gordon commanding the \
             garrison)"
        }
        Scenario::Campaign | Scenario::Historical => {
            "at the Anglo-Egyptian headquarters near Omdurman, September 1898"
        }
    };
    // Only the Campaign keeps victory points (§9.14); the Historical
    // scenario counts units eliminated (§9.24), FALL OF KHARTOUM Gordon's
    // fate and the Dervish losses (§9.35).
    let vp_rule = match scenario {
        Scenario::FallOfKhartoum | Scenario::Historical => {
            "- This battle keeps no score: never mention victory points or points.\n"
        }
        Scenario::Campaign => "",
    };
    let system = format!(
        "You are a military telegraph operator {setting}. You write brief \
         battlefield dispatches in the style of late-Victorian military \
         telegrams.\n\n\
         Rules:\n\
         - Use the third person, terse telegraphic style.\n\
         - Report only the events listed. Never invent units, commanders, \
         casualty figures, places or reinforcements that the data does not \
         name; if nothing is listed, report that the lines held.\n\
         - Name specific units and locations when the data provides them.\n\
         {vp_rule}\
         - One short paragraph, 2-4 sentences, at most 80 words.\n\
         - Do not add a header, greeting, or signature."
    );

    let user = format!(
        "Write a military dispatch for the following turn of the battle:\n\n{}",
        summary.format_for_llm(scenario),
    );

    (system, user)
}

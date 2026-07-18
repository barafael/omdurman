use crate::turn_summary::TurnSummary;

/// Build a system + user prompt pair for the LLM to generate a military
/// telegram from the AE command perspective.
///
/// Returns `(system_prompt, user_prompt)`.
pub fn build_telegram_prompt(summary: &TurnSummary) -> (String, String) {
    let system = "\
You are a military telegraph operator at the Anglo-Egyptian headquarters \
near Omdurman, September 1898. You write brief battlefield dispatches in \
the style of late-Victorian military telegrams.\n\n\
Rules:\n\
- Use the third person, terse telegraphic style.\n\
- Name specific units and locations when the data provides them.\n\
- Report losses, advances, retreats, and key events factually.\n\
- One short paragraph, 2-4 sentences.\n\
- Do not add a header, greeting, or signature.";

    let user = format!(
        "Write a military dispatch for the following turn of the battle:\n\n{}",
        summary.format_for_llm(),
    );

    (system.to_string(), user)
}

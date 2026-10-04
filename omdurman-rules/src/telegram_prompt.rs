use crate::turn_summary::{TurnEventRecord, TurnSummary};
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
    let vp_rule = if scenario.keeps_victory_points() {
        ""
    } else {
        "- This battle keeps no score: never mention victory points or points.\n"
    };
    let system = format!(
        "You are a military telegraph operator {setting}. You key field \
         telegrams in the manner of the 1880s and 1890s, when every word was \
         paid for.\n\n\
         Rules:\n\
         - ALL CAPITALS. Super terse: drop articles, pronouns and auxiliary \
         verbs (ENEMY ADVANCING KERRERI, not The enemy is advancing).\n\
         - No punctuation at all. End every sentence with the word STOP and \
         the whole message with FULL STOP.\n\
         - Report only the events listed. Never invent units, commanders, \
         casualty figures, places or reinforcements that the data does not \
         name; if nothing is listed, report ALL QUIET STOP LINES HOLD FULL STOP.\n\
         - Name units when the data provides them. Never quote hex \
         coordinates such as (12, 5): readers cannot place them.\n\
         {vp_rule}\
         - Two to five sentences, at most 40 words.\n\
         - No header, greeting or signature.\n\n\
         Example: DERVISH MASSING WEST STOP GUNBOATS ENGAGED BAGGARA STOP TWO \
         ENEMY BANDS DESTROYED STOP LINES HOLD FULL STOP"
    );

    let user = format!(
        "Write a military dispatch for the following turn of the battle:\n\n{}",
        summary.format_for_llm(scenario),
    );

    (system, user)
}

/// `text` keyed as a telegram of the day: all capitals, no punctuation,
/// every sentence ended by STOP and the message by FULL STOP. Applied to
/// whatever writes the telegram -- the flavour model, which may slip, the
/// fallback, and records filed before the style -- and idempotent.
pub fn telegraphese(text: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let end_sentence = |words: &mut Vec<String>| {
        if !words.is_empty() && words.last().is_some_and(|w| w != "STOP") {
            words.push("STOP".to_string());
        }
    };
    for c in text.chars() {
        match c {
            '.' | '!' | '?' | ';' | ':' | '\n' => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
                end_sentence(&mut words);
            }
            c if c.is_alphanumeric() || c == '\'' || c == '-' => {
                current.extend(c.to_uppercase());
            }
            _ => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    // FULL STOP inside the text (the model's) is a sentence end.
    let mut out: Vec<String> = Vec::new();
    for word in words {
        if word == "STOP" && out.last().is_some_and(|w| w == "FULL") {
            out.pop();
            if out.last().is_some_and(|w| w != "STOP") {
                out.push("STOP".into());
            }
            continue;
        }
        if word == "STOP" && (out.is_empty() || out.last().is_some_and(|w| w == "STOP")) {
            continue;
        }
        out.push(word);
    }
    while out.last().is_some_and(|w| w == "STOP") {
        out.pop();
    }
    if out.is_empty() {
        return "ALL QUIET STOP LINES HOLD FULL STOP".into();
    }
    out.push("FULL".into());
    out.push("STOP".into());
    out.join(" ")
}

/// A telegram keyed from the turn's own events, for when no flavour model
/// is available: from the British command's side (the Omdurman HQ, or
/// GORDON's palace at Khartoum), terse, with counts rather than lists.
pub fn fallback_telegram(summary: &TurnSummary) -> String {
    use omdurman_types::Player;
    let owner = |id: &crate::UnitId| {
        crate::unit_profiles::profile_for_unit(*id).map(|p| p.identity.owner())
    };
    let name = |id: &crate::UnitId| {
        crate::unit_profiles::profile_for_unit(*id)
            .map_or_else(|| "UNIT".to_string(), |p| p.identity.short_label())
    };
    let mut sentences: Vec<String> = Vec::new();
    let mut arrived = [0usize; 2]; // [ours, theirs]
    let mut lost_ours: Vec<String> = Vec::new();
    let mut lost_theirs = 0usize;
    let mut breaches = (0usize, 0usize); // (attempts, breached)
    let mut melees = 0usize;
    let mut deserted = 0usize;
    for event in &summary.events {
        match event {
            TurnEventRecord::Reinforcements { units, player, .. } => {
                arrived[usize::from(*player == Player::Dervish)] += units.len();
            }
            TurnEventRecord::UnitEliminated { unit, .. } => match owner(unit) {
                Some(Player::AngloEgyptian) => lost_ours.push(name(unit)),
                Some(Player::Dervish) => lost_theirs += 1,
                None => {}
            },
            TurnEventRecord::WallBreach { breached, .. } => {
                breaches.0 += 1;
                breaches.1 += usize::from(*breached);
            }
            TurnEventRecord::MeleeCombat { .. } => melees += 1,
            TurnEventRecord::Desertion { units, .. } => deserted += units.len(),
            _ => {}
        }
    }
    if arrived[0] > 0 {
        sentences.push(format!("{} OF OURS ARRIVED", arrived[0]));
    }
    if arrived[1] > 0 {
        sentences.push(format!("ENEMY REINFORCED {} BANDS", arrived[1]));
    }
    if melees > 0 {
        sentences.push(format!("HAND TO HAND FIGHTING {melees} PLACES"));
    }
    if lost_theirs > 0 {
        sentences.push(format!("{lost_theirs} ENEMY BANDS DESTROYED"));
    }
    if !lost_ours.is_empty() {
        sentences.push(format!("REGRET LOSS {}", lost_ours.join(" ")));
    }
    if deserted > 0 {
        sentences.push(format!("{deserted} ENEMY BANDS DESERTED IN NIGHT"));
    }
    match breaches {
        (0, _) => {}
        (_, 0) => sentences.push("ENEMY GUNS FAILED AGAINST WALL".into()),
        (_, n) => sentences.push(format!("WALL BREACHED {n} PLACES")),
    }
    if sentences.is_empty() {
        sentences.push("ALL QUIET".into());
    }
    sentences.push("LINES HOLD".into());
    telegraphese(&sentences.join(". "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telegraphese_keys_stops_and_capitals() {
        assert_eq!(
            telegraphese("The enemy advanced on Kerreri. Gunboats engaged, heavily!"),
            "THE ENEMY ADVANCED ON KERRERI STOP GUNBOATS ENGAGED HEAVILY FULL STOP"
        );
        // Idempotent, and the model's own STOP / FULL STOP are kept once.
        let keyed = "DERVISH MASSING STOP LINES HOLD FULL STOP";
        assert_eq!(telegraphese(keyed), keyed);
        assert_eq!(
            telegraphese("Lines hold STOP. Full stop."),
            "LINES HOLD FULL STOP"
        );
        assert_eq!(telegraphese(""), "ALL QUIET STOP LINES HOLD FULL STOP");
        // Apostrophes and unit codes survive.
        assert_eq!(
            telegraphese("Mahdi's Tomb held; 3E First Btn lost"),
            "MAHDI'S TOMB HELD STOP 3E FIRST BTN LOST FULL STOP"
        );
    }

    #[test]
    fn fallback_counts_rather_than_lists() {
        let summary = TurnSummary {
            turn: crate::GameTurnIndex::new(1),
            time: crate::turn_track::GameTime::SixAM,
            day_night: omdurman_types::DayNight::Day,
            first_player: omdurman_types::Player::AngloEgyptian,
            events: vec![
                TurnEventRecord::UnitEliminated {
                    unit: crate::UnitId::MulazminII_5_1,
                    cause: crate::effects::ElimCause::Combat,
                },
                TurnEventRecord::UnitEliminated {
                    unit: crate::UnitId::MulazminII_5_0,
                    cause: crate::effects::ElimCause::Combat,
                },
            ],
        };
        assert_eq!(
            fallback_telegram(&summary),
            "2 ENEMY BANDS DESTROYED STOP LINES HOLD FULL STOP"
        );
        let quiet = TurnSummary {
            events: vec![],
            ..summary
        };
        assert_eq!(
            fallback_telegram(&quiet),
            "ALL QUIET STOP LINES HOLD FULL STOP"
        );
    }
}

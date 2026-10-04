//! The press of the game, written deterministically from what happened: one
//! field telegram per finished turn ([`telegram`]) and, at the end, the
//! newspaper's front page ([`gazette`]). Every sentence is a fact from the
//! turn record or the game state, worded from the phrase banks in
//! [`phrases`] -- nothing is invented, and the same game always reads the
//! same.

pub mod gazette;
pub mod phrases;
pub mod telegram;

use std::sync::OnceLock;

use omdurman_types::{HexCoord, Scenario};

use crate::effects::GameState;

/// A phrase bank: interchangeable wordings of one fact.
pub type Bank = &'static [&'static str];

/// Pick one wording of `bank` for `salt` -- deterministic (the same game
/// reads the same everywhere), varied (different turns and facts get
/// different wordings).
pub fn pick(bank: Bank, salt: u64) -> &'static str {
    if bank.is_empty() {
        return "";
    }
    bank[(mix(salt) % bank.len() as u64) as usize]
}

/// A salt from the parts of what is being worded.
pub fn salt(parts: &[u64]) -> u64 {
    parts
        .iter()
        .fold(0x9E37_79B9_7F4A_7C15, |acc, &p| mix(acc ^ p))
}

/// splitmix64's finaliser: a cheap, well-spread hash of one word.
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// `template` with each `{key}` replaced by its value. Unknown keys stay.
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    sentence_case(&agree(out.trim()))
}

/// Every sentence begun with a capital (a placeholder may open one with a
/// number in words: "... severe. fourteen of our units"). A dot after an
/// abbreviation ("a.m.", "Sept.", "Hon.") ends no sentence.
pub fn sentence_case(text: &str) -> String {
    const ABBREVIATIONS: [&str; 8] = ["a.m", "p.m", "sept", "jan", "feb", "hon", "dr", "no"];
    let mut out = String::with_capacity(text.len());
    let mut start = true;
    let mut word = String::new();
    let mut ended = false;
    for c in text.chars() {
        if start && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            start = false;
        } else {
            out.push(c);
            if !c.is_whitespace() {
                start = false;
            }
        }
        match c {
            '.' | '!' | '?' => {
                ended = c != '.' || !ABBREVIATIONS.contains(&word.to_lowercase().as_str());
                word.push(c);
            }
            ' ' => {
                start = ended;
                ended = false;
                word.clear();
            }
            c => {
                ended = false;
                if c.is_alphanumeric() {
                    word.push(c);
                } else {
                    word.clear();
                }
            }
        }
    }
    out
}

/// `n` in words, as a telegram or a leader writer spells numbers below a
/// hundred ("thirty-four"); larger ones in figures.
pub fn number_word(n: usize) -> String {
    const UNITS: [&str; 20] = [
        "no",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    const TENS: [&str; 10] = [
        "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];
    match n {
        0..=19 => UNITS[n].to_string(),
        20..=99 if n.is_multiple_of(10) => TENS[n / 10].to_string(),
        20..=99 => format!("{}-{}", TENS[n / 10], UNITS[n % 10]),
        _ => n.to_string(),
    }
}

/// Number agreement after filling: "one bands" -> "one band", "one miles"
/// -> "one mile" (the banks are written for the plural).
pub fn agree(text: &str) -> String {
    let mut out = text.to_string();
    for (plural, singular) in [
        ("bands", "band"),
        ("miles", "mile"),
        ("units", "unit"),
        ("hours", "hour"),
        ("times", "time"),
        ("occasions", "occasion"),
    ] {
        for one in ["one", "One", "ONE"] {
            out = out.replace(&format!("{one} {plural}"), &format!("{one} {singular}"));
            out = out.replace(
                &format!("{one} enemy {plural}"),
                &format!("{one} enemy {singular}"),
            );
        }
    }
    out
}

/// What the telegraph clerk keys for a unit: "3E First Btn", "battery",
/// "squadron", "Gunboat Naser", "General Gatacre"; a tribe by name.
pub fn wire_name(id: crate::UnitId, scenario: Scenario) -> String {
    use crate::UnitIdentity as U;
    let Some(identity) = crate::unit_profiles::profile_for_unit(id).map(|p| p.identity) else {
        return "unit".into();
    };
    match identity {
        // Kitchener commanded as the Sirdar (his counter: "Lord Kitchener,
        // Sirdar"); Gordon was Governor-General.
        U::AngloEgyptianLeader(crate::BritishLeader::Kitchener) => "the Sirdar".into(),
        U::AngloEgyptianLeader(l) => format!("General {l}"),
        U::AngloEgyptianArtillery => "battery".into(),
        U::AngloEgyptianMaxim => "Maxim battery".into(),
        U::AngloEgyptianCavalry => "squadron".into(),
        U::AngloEgyptianCamelCorps => "camel company".into(),
        U::RoyalEngineers => "Engineers".into(),
        U::AngloEgyptianInfantry { .. } if identity.is_friendlies() => "Friendlies".into(),
        other => other.label_in(scenario),
    }
}

/// What a correspondent calls one of our units in prose: "General
/// Gatacre", "the gunboat Naser", "a field battery", "the First
/// Battalion of the 3E Brigade"; an enemy unit by its tribe or leader.
pub fn prose_name(id: crate::UnitId, scenario: Scenario) -> String {
    use crate::UnitIdentity as U;
    let Some(identity) = crate::unit_profiles::profile_for_unit(id).map(|p| p.identity) else {
        return "a unit".into();
    };
    match identity {
        // Kitchener commanded as the Sirdar (his counter: "Lord Kitchener,
        // Sirdar"); Gordon was Governor-General.
        U::AngloEgyptianLeader(crate::BritishLeader::Kitchener) => "the Sirdar".into(),
        U::AngloEgyptianLeader(l) => format!("General {l}"),
        U::AngloEgyptianGunboat(_) => {
            // "the gunboat Sultan", "the steamer Bordein" (1885).
            let label = identity.label_in(scenario);
            match label.split_once(' ') {
                Some((kind, name)) => format!("the {} {name}", kind.to_lowercase()),
                None => label,
            }
        }
        U::AngloEgyptianArtillery => "a field battery".into(),
        U::AngloEgyptianMaxim => "a Maxim battery".into(),
        U::AngloEgyptianCavalry => "a squadron of cavalry".into(),
        U::AngloEgyptianCamelCorps => "a company of the Camel Corps".into(),
        U::RoyalEngineers => "a company of Royal Engineers".into(),
        U::AngloEgyptianFort => "a fort".into(),
        U::AngloEgyptianInfantry { .. } if identity.is_friendlies() => {
            "a body of Friendlies".into()
        }
        U::AngloEgyptianInfantry { brigade, battalion } => {
            format!("the {battalion} Battalion of the {brigade} Brigade")
        }
        other => other.label_in(scenario),
    }
}

/// `n` of the unit kind `id` belongs to, in prose: "six field batteries",
/// "five bodies of Friendlies"; [`prose_name`] for one.
pub fn prose_count(id: crate::UnitId, n: usize, scenario: Scenario) -> String {
    use crate::UnitIdentity as U;
    if n == 1 {
        return prose_name(id, scenario);
    }
    let count = number_word(n);
    match crate::unit_profiles::profile_for_unit(id).map(|p| p.identity) {
        Some(U::AngloEgyptianArtillery) => format!("{count} field batteries"),
        Some(U::AngloEgyptianMaxim) => format!("{count} Maxim batteries"),
        Some(U::AngloEgyptianCavalry) => format!("{count} squadrons of cavalry"),
        Some(U::AngloEgyptianCamelCorps) => format!("{count} companies of the Camel Corps"),
        Some(U::RoyalEngineers) => format!("{count} companies of Royal Engineers"),
        Some(identity @ U::AngloEgyptianInfantry { .. }) if identity.is_friendlies() => {
            format!("{count} bodies of Friendlies")
        }
        _ => format!("{} ({count})", prose_name(id, scenario)),
    }
}

/// Every elimination of a turn with where it happened: the place of the
/// last combat before it in the record -- fire, melee, a shell, a breach --
/// since not every resolution records its own hex.
pub fn deaths(
    summary: &crate::turn_summary::TurnSummary,
) -> Vec<(crate::UnitId, Option<HexCoord>)> {
    use crate::turn_summary::TurnEventRecord as E;
    // A combat record names the units it killed (it is pushed after their
    // eliminations); otherwise the last combat before the elimination.
    let mut named: Vec<(crate::UnitId, HexCoord)> = Vec::new();
    for event in &summary.events {
        match event {
            E::FireCombat {
                target, eliminated, ..
            } => named.extend(eliminated.iter().map(|&u| (u, *target))),
            E::MeleeCombat {
                hex,
                attacker_losses,
                defender_losses,
                ..
            } => named.extend(
                attacker_losses
                    .iter()
                    .chain(defender_losses)
                    .map(|&u| (u, *hex)),
            ),
            _ => {}
        }
    }
    let mut last: Option<HexCoord> = None;
    let mut out = Vec::new();
    for event in &summary.events {
        match event {
            E::FireCombat { target, .. } => last = Some(*target),
            E::MeleeCombat { hex, .. } => last = Some(*hex),
            E::HowitzerImpact { at, .. } => last = Some(*at),
            E::WallBreach { hexside, .. } => last = Some(hexside.a),
            E::UnitEliminated { unit, .. } => {
                let at = named
                    .iter()
                    .find(|(u, _)| u == unit)
                    .map(|(_, h)| *h)
                    .or(last);
                out.push((*unit, at));
            }
            _ => {}
        }
    }
    out
}

/// Where the turn's first combat took place, if there was one (`Some(None)`
/// when the record has a casualty but no hex for it).
pub fn first_combat(summary: &crate::turn_summary::TurnSummary) -> Option<Option<HexCoord>> {
    use crate::turn_summary::TurnEventRecord as E;
    let fought = summary.events.iter().any(|event| {
        matches!(
            event,
            E::FireCombat { .. }
                | E::MeleeCombat { .. }
                | E::HowitzerImpact { .. }
                | E::UnitEliminated { .. }
        )
    });
    if !fought {
        return None;
    }
    // An elimination is recorded before the combat that caused it; the first
    // combat with a hex is where the fighting began.
    let at = summary.events.iter().find_map(|event| match event {
        E::FireCombat { target, .. } => Some(*target),
        E::MeleeCombat { hex, .. } => Some(*hex),
        E::HowitzerImpact { at, .. } => Some(*at),
        _ => None,
    });
    Some(at.or_else(|| deaths(summary).into_iter().find_map(|(_, at)| at)))
}

/// `text` with its first letter upper-cased.
pub fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// `items` as English prose: "a", "a and b", "a, b and c".
pub fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// The landmarks printed on the Campaign map that are not hex names in the
/// board data (read off the map scan against the hex calibration).
const CAMPAIGN_PRINTED: &[(&str, (i32, i32))] = &[
    ("Abu Ledat", (6, 4)),
    ("El Azrak", (11, 4)),
    ("Um Matragan", (17, 6)),
    ("the Kerreri Hills", (25, 7)),
    ("Jebel Surgham", (31, 18)),
    ("the Khor Shambat", (16, 16)),
];

/// A named place: the hex at its middle, its name, and every hex it covers.
type Place = (HexCoord, String, Vec<HexCoord>);

/// Every named place of a scenario's board: the board data's hex names (one
/// entry per name) plus the printed landmarks.
fn places(scenario: Scenario) -> &'static [Place] {
    static CAMPAIGN: OnceLock<Vec<Place>> = OnceLock::new();
    static KHARTOUM: OnceLock<Vec<Place>> = OnceLock::new();
    let build = |map: omdurman_types::MapData, printed: &[(&str, (i32, i32))]| {
        let mut by_name: std::collections::BTreeMap<String, Vec<HexCoord>> = Default::default();
        for ((q, r), tile) in &map.tiles {
            if let Some(name) = tile.name.as_deref().filter(|n| !n.is_empty()) {
                // Bare landmark names read as places; a few need an article.
                let name = match name {
                    "Zariba" => "the Zariba",
                    "Grounds" | "Treasury" | "Arsenal" => continue,
                    "Palace" => "the Palace",
                    other => other,
                };
                by_name
                    .entry(name.to_string())
                    .or_default()
                    .push(HexCoord::new(*q, *r));
            }
        }
        let mut out: Vec<Place> = by_name
            .into_iter()
            .map(|(name, hexes)| {
                // The hex nearest the middle of the named area.
                let (sq, sr) = hexes.iter().fold((0, 0), |(a, b), h| (a + h.q, b + h.r));
                let n = hexes.len() as i32;
                let mid = HexCoord::new(sq / n, sr / n);
                let at = *hexes
                    .iter()
                    .min_by_key(|h| h.distance(mid))
                    .expect("a named area has hexes");
                (at, name, hexes)
            })
            .collect();
        out.extend(printed.iter().map(|(name, (q, r))| {
            let at = HexCoord::new(*q, *r);
            (at, (*name).to_string(), vec![at])
        }));
        out
    };
    match scenario {
        Scenario::FallOfKhartoum => {
            KHARTOUM.get_or_init(|| build(crate::board_data::fall_of_khartoum_map_data(), &[]))
        }
        Scenario::Campaign | Scenario::Historical => {
            CAMPAIGN.get_or_init(|| build(crate::board_data::campaign_map_data(), CAMPAIGN_PRINTED))
        }
    }
}

/// The city the dispatches measure distances from.
fn home_city(scenario: Scenario) -> &'static str {
    match scenario {
        Scenario::FallOfKhartoum => "Khartoum",
        Scenario::Campaign | Scenario::Historical => "Omdurman",
    }
}

/// Where `hex` lies, as a correspondent writes it: "near Kerreri", "at
/// Omdurman", or "three miles north of Omdurman" (a hex is about 400
/// yards, §1.2).
pub fn whereabouts(scenario: Scenario, hex: HexCoord) -> String {
    let places = places(scenario);
    if let Some((_, name, _)) = places.iter().find(|(_, _, hexes)| hexes.contains(&hex)) {
        return format!("at {name}");
    }
    if let Some((_, name, _)) = places
        .iter()
        .filter(|(at, _, _)| at.distance(hex) <= 3)
        .min_by_key(|(at, _, _)| at.distance(hex))
    {
        return format!("near {name}");
    }
    let city = home_city(scenario);
    let Some((centre, _, _)) = places.iter().find(|(_, name, _)| name == city) else {
        return "in the field".into();
    };
    let miles = ((hex.distance(*centre) + 2) / 4).max(1) as usize;
    let dq = (hex.q - centre.q) as f32 + (hex.r - centre.r) as f32 / 2.0;
    let dr = (hex.r - centre.r) as f32;
    // Bearing clockwise from north (rows run north to south).
    let bearing = (dq * 0.866)
        .atan2(-dr * 0.75)
        .to_degrees()
        .rem_euclid(360.0);
    const POINTS: [&str; 8] = [
        "north",
        "north-east",
        "east",
        "south-east",
        "south",
        "south-west",
        "west",
        "north-west",
    ];
    let point = POINTS[((bearing + 22.5) / 45.0) as usize % 8];
    let miles_word = if miles == 1 {
        "a mile".to_string()
    } else {
        format!("{} miles", number_word(miles))
    };
    format!("{miles_word} {point} of {city}")
}

/// The date of a turn as the press prints it: "September 2" and the hour
/// ("6 a.m."), read off the scenario's Turn Record Track (the Campaign's
/// nights are two turns, so the clock is not a uniform two hours a turn).
pub fn turn_date(scenario: Scenario, turn: u8) -> (String, String) {
    use crate::turn_track::GameTime as T;
    let (day0, month) = match scenario {
        Scenario::Campaign => (1u32, "September"),
        Scenario::Historical => (2, "September"),
        Scenario::FallOfKhartoum => (26, "January"),
    };
    let hour = |time: T| -> u32 {
        match time {
            T::SixAM => 6,
            T::EightAM => 8,
            T::TenAM => 10,
            T::Noon => 12,
            T::TwoPM => 14,
            T::FourPM => 16,
            T::SixPM => 18,
            T::EightPM => 20,
            T::TenPM => 22,
            T::Midnight => 24,
            T::TwoAM => 26,
            T::FourAM => 28,
        }
    };
    // Walk the track on a running clock (hours since midnight of the first
    // day): each turn's hour of day, moved on a day whenever it would not
    // come after the turn before -- "midnight" after "10 p.m." is the next
    // day, so is "6 a.m." after it.
    let mut elapsed: Option<u32> = None;
    for t in 1..=turn.max(1) {
        let Some(entry) = crate::turn_track::scenario_turn(scenario, crate::GameTurnIndex::new(t))
        else {
            break;
        };
        let of_day = hour(entry.time) % 24;
        elapsed = Some(match elapsed {
            None => of_day,
            Some(before) => {
                let mut now = before - before % 24 + of_day;
                while now <= before {
                    now += 24;
                }
                now
            }
        });
    }
    let elapsed = elapsed.unwrap_or(6);
    let day = day0 + elapsed / 24;
    let at = elapsed % 24;
    let clock = match at {
        0 => "midnight".to_string(),
        12 => "noon".to_string(),
        h if h < 12 => format!("{h} a.m."),
        h => format!("{} p.m.", h - 12),
    };
    (format!("{month} {day}"), clock)
}

/// The year of a scenario's battle.
pub fn year(scenario: Scenario) -> u16 {
    match scenario {
        Scenario::FallOfKhartoum => 1885,
        Scenario::Campaign | Scenario::Historical => 1898,
    }
}

/// The hex nearest the middle of `hexes` (a body of troops), if any.
pub fn centre_of(hexes: &[HexCoord]) -> Option<HexCoord> {
    if hexes.is_empty() {
        return None;
    }
    let (sq, sr) = hexes
        .iter()
        .fold((0i64, 0i64), |(a, b), h| (a + h.q as i64, b + h.r as i64));
    let n = hexes.len() as i64;
    let mid = HexCoord::new((sq / n) as i32, (sr / n) as i32);
    hexes.iter().copied().min_by_key(|h| h.distance(mid))
}

/// The hexes of `player`'s field force: units that move (forts and
/// gunboats are not where an army stands).
pub fn field_force(state: &GameState, player: omdurman_types::Player) -> Vec<HexCoord> {
    state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == player)
        .filter(|u| !u.profile.kind.is_boat())
        .filter(|u| !matches!(u.profile.movement, crate::UnitMovement::Immobile))
        .map(|u| u.position)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_name_the_map() {
        // The Kerreri huts and a printed landmark, and the miles fallback.
        assert_eq!(
            whereabouts(Scenario::Campaign, HexCoord::new(30, 5)),
            "at Kerreri"
        );
        assert_eq!(
            whereabouts(Scenario::Campaign, HexCoord::new(31, 19)),
            "near Jebel Surgham"
        );
        let far = whereabouts(Scenario::Campaign, HexCoord::new(5, 30));
        assert!(
            far.ends_with("of Omdurman") || far.starts_with("near"),
            "{far}"
        );
    }

    #[test]
    fn turns_carry_their_date_and_hour() {
        assert_eq!(
            turn_date(Scenario::Campaign, 1),
            ("September 1".to_string(), "6 a.m.".to_string())
        );
        // The Campaign's nights are two turns: midnight, then dawn.
        assert_eq!(
            turn_date(Scenario::Campaign, 10),
            ("September 2".to_string(), "midnight".to_string())
        );
        assert_eq!(
            turn_date(Scenario::Campaign, 11),
            ("September 2".to_string(), "6 a.m.".to_string())
        );
        assert_eq!(
            turn_date(Scenario::Campaign, 22),
            ("September 3".to_string(), "8 a.m.".to_string())
        );
        assert_eq!(
            turn_date(Scenario::Historical, 4),
            ("September 2".to_string(), "noon".to_string())
        );
        assert_eq!(
            turn_date(Scenario::FallOfKhartoum, 1),
            ("January 26".to_string(), "4 a.m.".to_string())
        );
    }

    #[test]
    fn sentences_begin_with_capitals_abbreviations_do_not_end_them() {
        assert_eq!(
            sentence_case("severe. fourteen units fell at 8 a.m. on Sept. 2. it was hard"),
            "Severe. Fourteen units fell at 8 a.m. on Sept. 2. It was hard"
        );
    }

    #[test]
    fn picking_is_deterministic_and_varied() {
        let bank: Bank = &["a", "b", "c", "d"];
        assert_eq!(pick(bank, salt(&[1, 2])), pick(bank, salt(&[1, 2])));
        let distinct: std::collections::BTreeSet<&str> =
            (0..32).map(|i| pick(bank, salt(&[i]))).collect();
        assert!(distinct.len() > 1);
    }
}

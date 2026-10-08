//! The newspaper's front page at the end of the game: the lead article on
//! the battle, written from the game's record; the turn telegrams as "Late
//! Telegrams"; a roll of honour; and the other news of the day (Fashoda and
//! the Dreyfus affair in 1898, the Desert Column and the Westminster
//! outrages in 1885), all from the phrase banks.

use omdurman_types::{HexCoord, Player, Scenario};

use super::phrases::{ADVERTS, FEATURES_1885, FEATURES_1898, LEAD as L, TELEGRAM_HEADS};
use super::{Bank, and_list, capitalize, fill, number_word, pick, salt, turn_date, whereabouts};
use crate::effects::GameState;
use crate::turn_summary::TurnEventRecord;
use crate::{GameResult, UnitId, UnitIdentity, VpSource};

/// One article: its head, the stacked decks under it, the dateline that
/// opens the first paragraph, and the paragraphs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Article {
    pub head: String,
    pub decks: Vec<String>,
    pub dateline: Option<String>,
    pub paragraphs: Vec<String>,
}

/// A small advertisement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Advert {
    pub head: String,
    pub lines: Vec<String>,
}

/// The whole front page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontPage {
    pub masthead: String,
    /// "No. 35,612"
    pub issue: String,
    /// "Saturday, September 3, 1898"
    pub date: String,
    pub price: String,
    /// The banner headline across the page.
    pub headline: String,
    pub lead: Article,
    /// The course of the battle, turn by turn: (heading "September 2,
    /// 8 a.m.", what happened in prose). Turns with nothing to report are
    /// left out.
    pub chronicle: Vec<(String, String)>,
    /// The last few turns' telegrams: (heading "September 1, 6 a.m.",
    /// telegram).
    pub telegrams: Vec<(String, String)>,
    /// Our units lost, by name.
    pub roll_of_honour: Vec<String>,
    /// The other news of the day.
    pub features: Vec<Article>,
    pub adverts: Vec<Advert>,
}

/// How the battle ended, as the leader writer sees it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Outcome {
    Won,
    Drawn,
    Lost,
    GordonSaved,
    GordonFellAvenged,
    GordonFell,
}

fn outcome(result: GameResult, gordon_fell: bool) -> Outcome {
    match result {
        GameResult::Campaign(level) => match level {
            crate::CampaignVictoryLevel::Draw => Outcome::Drawn,
            crate::CampaignVictoryLevel::Marginal(p)
            | crate::CampaignVictoryLevel::Tactical(p)
            | crate::CampaignVictoryLevel::Decisive(p) => {
                if p == Player::AngloEgyptian {
                    Outcome::Won
                } else {
                    Outcome::Lost
                }
            }
        },
        GameResult::Historical { ae, d } => match crate::HistoricalVictoryLevel::net(ae, d).0 {
            None => Outcome::Drawn,
            Some(Player::AngloEgyptian) => Outcome::Won,
            Some(Player::Dervish) => Outcome::Lost,
        },
        GameResult::FoK(level) => match (gordon_fell, level as i8 > 0) {
            (false, _) => Outcome::GordonSaved,
            (true, true) => Outcome::GordonFellAvenged,
            (true, false) => Outcome::GordonFell,
        },
    }
}

/// What the record says happened, gathered once.
#[derive(Default)]
struct Record {
    /// (turn, hex) of the first combat (the hex when the record has one).
    first_contact: Option<(u8, Option<HexCoord>)>,
    /// (turn, enemy units lost that turn, where they fell).
    bloodiest: Option<(u8, usize, Option<HexCoord>)>,
    melees: usize,
    /// (turn, complete) when the Zariba was finished, else first built.
    zariba: Option<(u8, bool)>,
    /// (hex, by our guns) of each breach made.
    breaches: Vec<(HexCoord, bool)>,
    /// Dervish emirs (leaders other than the Khalifa) killed.
    emirs: Vec<UnitId>,
    tomb_taken: Option<u8>,
    khalifa_at: Option<Option<HexCoord>>,
    deserted: usize,
    ours_lost: Vec<UnitId>,
    /// Every enemy loss (leaders, bands, works); `bands` counts the tribes.
    theirs_lost: Vec<UnitId>,
    bands: usize,
    /// The enemy's forts, guns and steamers destroyed.
    works: Vec<UnitId>,
}

fn identity(id: UnitId) -> Option<UnitIdentity> {
    crate::unit_profiles::profile_for_unit(id).map(|p| p.identity)
}

fn read_record(state: &GameState) -> Record {
    let mut rec = Record::default();
    for summary in &state.turn_summaries {
        let turn = summary.turn.value();
        if rec.first_contact.is_none()
            && let Some(at) = super::first_combat(summary)
        {
            rec.first_contact = Some((turn, at));
        }
        for event in &summary.events {
            match event {
                TurnEventRecord::MeleeCombat { .. } => rec.melees += 1,
                TurnEventRecord::ZaribaBuilt { complete, .. } => {
                    // When begun, until the record says it was finished.
                    if rec.zariba.is_none_or(|(_, done)| *complete && !done) {
                        rec.zariba = Some((turn, *complete));
                    }
                }
                TurnEventRecord::WallBreach {
                    attacker,
                    hexside,
                    breached: true,
                    ..
                } => rec
                    .breaches
                    .push((hexside.a, *attacker == Player::AngloEgyptian)),
                TurnEventRecord::VpScored {
                    source: VpSource::MahdisTombTaken,
                    ..
                } => {
                    rec.tomb_taken.get_or_insert(turn);
                }
                TurnEventRecord::Desertion { units, .. } => rec.deserted += units.len(),
                _ => {}
            }
        }
        let mut theirs_this_turn = 0usize;
        let mut their_places: Vec<HexCoord> = Vec::new();
        for (unit, at) in super::deaths(summary) {
            match identity(unit) {
                Some(UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah)) => {
                    rec.khalifa_at = Some(at);
                    rec.theirs_lost.push(unit);
                }
                Some(UnitIdentity::DervishLeader(_)) => {
                    rec.emirs.push(unit);
                    rec.theirs_lost.push(unit);
                }
                Some(i) if i.owner() == Player::AngloEgyptian => rec.ours_lost.push(unit),
                Some(UnitIdentity::DervishTribal { .. }) => {
                    rec.theirs_lost.push(unit);
                    rec.bands += 1;
                    theirs_this_turn += 1;
                    their_places.extend(at);
                }
                Some(_) => {
                    rec.theirs_lost.push(unit);
                    rec.works.push(unit);
                }
                None => {}
            }
        }
        if theirs_this_turn >= 2
            && rec
                .bloodiest
                .as_ref()
                .is_none_or(|(_, best, _)| theirs_this_turn > *best)
        {
            rec.bloodiest = Some((turn, theirs_this_turn, super::centre_of(&their_places)));
        }
    }
    rec
}

/// The enemy's lost forts, guns and steamers in prose: "a fort, two
/// batteries and a steamer".
fn works_prose(works: &[UnitId]) -> String {
    let count = |pick: fn(&UnitIdentity) -> bool| {
        works
            .iter()
            .filter(|&&u| identity(u).is_some_and(|i| pick(&i)))
            .count()
    };
    let kinds = [
        (
            count(|i| matches!(i, UnitIdentity::DervishFort)),
            "a fort",
            "forts",
        ),
        (
            count(|i| matches!(i, UnitIdentity::DervishArtillery)),
            "a battery",
            "batteries",
        ),
        (
            count(|i| matches!(i, UnitIdentity::DervishGunboat(_))),
            "a steamer",
            "steamers",
        ),
    ];
    let parts: Vec<String> = kinds
        .iter()
        .filter(|(n, _, _)| *n > 0)
        .map(|&(n, one, many)| {
            if n == 1 {
                one.to_string()
            } else {
                format!("{} {many}", number_word(n))
            }
        })
        .collect();
    and_list(&parts)
}

/// The day-of-week of a Gregorian date (Zeller's congruence).
fn weekday(year: i32, month: u32, day: u32) -> &'static str {
    let (y, m) = if month < 3 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let k = y % 100;
    let j = y / 100;
    let h = (day as i32 + 13 * (m as i32 + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    [
        "Saturday",
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
    ][h as usize]
}

/// The front page for a finished game. `telegrams` are the turn telegrams
/// as filed (turn, text), for the Late Telegrams column.
pub fn front_page(state: &GameState, telegrams: &[(u8, String)]) -> FrontPage {
    let scenario = state.scenario;
    let result = state
        .game_result
        .unwrap_or(GameResult::Campaign(crate::CampaignVictoryLevel::Draw));
    let gordon_fell = state.gordon_eliminated_turn.is_some();
    let template = crate::newspaper::newspaper_template(result, gordon_fell);
    let outcome = outcome(result, gordon_fell);
    let rec = read_record(state);
    let last_turn = state.turn_summaries.last().map_or(1, |s| s.turn.value());
    let city = match scenario {
        Scenario::FallOfKhartoum => "Khartoum",
        Scenario::Campaign | Scenario::Historical => "Omdurman",
    };
    let game_salt = salt(&[
        scenario as u64,
        u64::from(last_turn),
        rec.theirs_lost.len() as u64,
    ]);
    let say = |bank: Bank, key: u64, values: &[(&str, &str)]| -> String {
        fill(pick(bank, salt(&[game_salt, key])), values)
    };
    let when = |turn: u8| turn_date(scenario, turn);
    let place_or_field = |at: Option<HexCoord>| {
        at.map_or_else(|| "on the field".into(), |h| whereabouts(scenario, h))
    };

    // -- the lead article -----------------------------------------------
    let hours = number_word(usize::from(last_turn) * 2);
    let mut paragraphs: Vec<String> = Vec::new();
    let opening = match outcome {
        Outcome::Won => L.opening_won,
        Outcome::Drawn => L.opening_drawn,
        Outcome::Lost => L.opening_lost,
        Outcome::GordonSaved => L.opening_gordon_saved,
        Outcome::GordonFellAvenged => L.opening_gordon_fell_avenged,
        Outcome::GordonFell => L.opening_gordon_fell,
    };
    let mut opening_text = say(opening, 1, &[("city", city), ("n", &hours)]);
    if outcome == Outcome::Drawn {
        let (theirs, ours) = (rec.theirs_lost.len(), rec.ours_lost.len());
        if theirs >= ours * 3 + 5 {
            opening_text = format!("{opening_text} {}", say(L.drawn_but_ahead, 16, &[]));
        } else if ours >= theirs + 5 {
            opening_text = format!("{opening_text} {}", say(L.drawn_but_behind, 16, &[]));
        }
    }
    paragraphs.push(opening_text);

    // The course of the battle.
    let mut course: Vec<String> = Vec::new();
    match rec.first_contact {
        Some((turn, at)) => {
            let (date, time) = when(turn);
            course.push(say(
                L.first_contact,
                2,
                &[
                    ("date", &date),
                    ("time", &time),
                    ("place", &place_or_field(at)),
                ],
            ));
        }
        None => course.push(say(L.no_contact, 2, &[])),
    }
    if let Some((turn, n, at)) = rec.bloodiest {
        let (date, time) = when(turn);
        course.push(say(
            L.bloodiest,
            3,
            &[
                ("date", &date),
                ("time", &time),
                ("n", &number_word(n)),
                ("place", &place_or_field(at)),
            ],
        ));
    }
    if let Some((turn, complete)) = rec.zariba {
        let (date, time) = when(turn);
        let bank = if complete {
            L.zariba_complete
        } else {
            L.zariba_begun
        };
        course.push(say(bank, 18, &[("date", &date), ("time", &time)]));
    }
    if rec.melees > 0 {
        let times = match rec.melees {
            1 => "once".to_string(),
            2 => "twice".to_string(),
            n => format!("{} times", number_word(n)),
        };
        // "once" reads better than "one time".
        let line = say(L.melees, 4, &[("n", &number_word(rec.melees))]).replace(
            &super::agree(&format!("{} times", number_word(rec.melees))),
            &times,
        );
        course.push(capitalize(&line));
    }
    if let Some(&(at, ours)) = rec.breaches.first() {
        let bank = if ours {
            L.we_breached
        } else {
            L.enemy_breached
        };
        course.push(say(bank, 5, &[("place", &whereabouts(scenario, at))]));
    }
    if let Some(turn) = rec.tomb_taken {
        let (date, time) = when(turn);
        course.push(say(L.tomb_taken, 6, &[("date", &date), ("time", &time)]));
    }
    if let Some(at) = rec.khalifa_at {
        course.push(say(L.khalifa_killed, 7, &[("place", &place_or_field(at))]));
    }
    if !rec.emirs.is_empty() {
        let names: Vec<String> = rec
            .emirs
            .iter()
            .map(|&u| super::prose_name(u, scenario))
            .collect();
        let bank = if names.len() == 1 {
            L.emir_killed
        } else {
            L.emirs_killed
        };
        course.push(say(bank, 17, &[("units", &and_list(&names))]));
    }
    if let Some(turn) = state.gordon_eliminated_turn {
        let (date, time) = when(turn.value());
        course.push(say(L.gordon_fell, 8, &[("date", &date), ("time", &time)]));
    }
    if rec.deserted > 0 {
        course.push(say(L.desertion, 9, &[("n", &number_word(rec.deserted))]));
    }
    paragraphs.push(course.join(" "));

    // Losses in words.
    let mut losses: Vec<String> = Vec::new();
    let alive = |player: Player| {
        state
            .units
            .iter()
            .filter(|u| u.profile.identity.owner() == player)
            .count()
    };
    let share = |lost: usize, alive: usize| lost as f32 / (lost + alive).max(1) as f32;
    let their_bank = match share(rec.theirs_lost.len(), alive(Player::Dervish)) {
        s if s >= 0.4 => L.enemy_losses_heavy,
        s if s >= 0.15 => L.enemy_losses_moderate,
        _ => L.enemy_losses_light,
    };
    if rec.bands > 0 {
        losses.push(capitalize(&say(
            their_bank,
            10,
            &[("n", &number_word(rec.bands))],
        )));
    }
    if !rec.works.is_empty() {
        losses.push(say(
            L.enemy_works,
            19,
            &[("works", &works_prose(&rec.works))],
        ));
    }
    let leaders: Vec<String> = rec
        .ours_lost
        .iter()
        .filter(|&&u| matches!(identity(u), Some(UnitIdentity::AngloEgyptianLeader(_))))
        .map(|&u| super::prose_name(u, scenario))
        .collect();
    let gunboats: Vec<String> = rec
        .ours_lost
        .iter()
        .filter(|&&u| matches!(identity(u), Some(UnitIdentity::AngloEgyptianGunboat(_))))
        .map(|&u| super::prose_name(u, scenario))
        .collect();
    let named: Vec<String> = {
        let mut seen: Vec<String> = Vec::new();
        for n in rec
            .ours_lost
            .iter()
            .map(|&u| super::prose_name(u, scenario))
        {
            if !seen.contains(&n) {
                seen.push(n);
            }
        }
        seen.truncate(3);
        seen
    };
    let ours_bank = match (
        rec.ours_lost.len(),
        share(rec.ours_lost.len(), alive(Player::AngloEgyptian)),
    ) {
        (0, _) => L.our_losses_none,
        (_, s) if s >= 0.3 => L.our_losses_heavy,
        (_, s) if s >= 0.1 => L.our_losses_moderate,
        _ => L.our_losses_light,
    };
    losses.push(say(
        ours_bank,
        11,
        &[
            ("n", &number_word(rec.ours_lost.len())),
            ("units", &and_list(&named)),
        ],
    ));
    if !leaders.is_empty() && scenario != Scenario::FallOfKhartoum {
        losses.push(say(L.leaders_lost, 12, &[("units", &and_list(&leaders))]));
    }
    if !gunboats.is_empty() {
        losses.push(say(L.gunboats, 13, &[("units", &and_list(&gunboats))]));
    }
    paragraphs.push(losses.join(" "));

    let closing = match outcome {
        Outcome::Won => L.closing_won,
        Outcome::Drawn => L.closing_drawn,
        Outcome::Lost => L.closing_lost,
        Outcome::GordonSaved => L.closing_gordon_saved,
        Outcome::GordonFellAvenged | Outcome::GordonFell => L.closing_gordon_fell,
    };
    paragraphs.push(say(closing, 14, &[("city", city)]));

    let (last_date, _) = when(last_turn);
    let dateline = format!(
        "{}, {}.\u{2014}",
        city.to_uppercase(),
        last_date
            .replace("September", "Sept.")
            .replace("January", "Jan.")
    );
    let deck = say(
        L.deck_losses,
        15,
        &[
            ("theirs", &deck_number(rec.bands)),
            ("ours", &deck_number(rec.ours_lost.len())),
        ],
    )
    // "None" stands alone ("Our Loss None"); before its noun it is "No".
    .replace("None Dervish Bands", "No Dervish Bands")
    .replace("Loses None Bands", "Loses No Bands");
    let lead = Article {
        head: battle_head(scenario).to_string(),
        decks: vec![template.subhead.to_string(), deck],
        dateline: Some(dateline),
        paragraphs,
    };

    // -- the masthead's date: the morning after, or when the news came ----
    let (date, issue) = match scenario {
        // The news of Khartoum reached London on February 5.
        Scenario::FallOfKhartoum => ("Thursday, February 5, 1885".to_string(), "No. 31,362"),
        Scenario::Campaign | Scenario::Historical => {
            let mut day: u32 = last_date
                .trim_start_matches("September ")
                .parse()
                .unwrap_or(2)
                + 1;
            // The dailies did not print on a Sunday.
            if weekday(1898, 9, day) == "Sunday" {
                day += 1;
            }
            (
                format!("{}, September {day}, 1898", weekday(1898, 9, day)),
                "No. 35,612",
            )
        }
    };

    // -- the course of the battle, turn by turn -------------------------
    let chronicle = state
        .turn_summaries
        .iter()
        .filter_map(|summary| {
            let sentences = super::telegram::chronicle(state, summary);
            if sentences.is_empty() {
                return None;
            }
            let (date, time) = when(summary.turn.value());
            Some((
                format!("{date}, {time}"),
                format!("{}.", sentences.join(". ")),
            ))
        })
        .collect();

    // -- late telegrams, roll of honour, the other news -----------------
    // The latest telegrams only: the chronicle tells the whole battle.
    const LATEST: usize = 3;
    let telegram_column = telegrams[telegrams.len().saturating_sub(LATEST)..]
        .iter()
        .map(|(turn, text)| {
            let (date, time) = when(*turn);
            // 1885: Gordon's messages came by runner, the news of the fall
            // by telegraph from Korti (see `telegram::filed`).
            use super::telegram::Filed;
            let head = match super::telegram::filed(state, *turn) {
                Filed::RunnerFromKhartoum => format!("{date}, {time} (by runner)"),
                Filed::TelegraphFromKorti => format!("Korti, {date} (by telegraph)"),
                Filed::FieldTelegraph => fill(
                    pick(TELEGRAM_HEADS, salt(&[game_salt, u64::from(*turn), 99])),
                    &[("date", &date), ("time", &time)],
                ),
            };
            (head, super::telegram::telegraphese(text))
        })
        .collect();
    // Each kind once, with how many fell: "Six field batteries".
    let roll_of_honour = {
        let mut counted: Vec<(String, UnitId, usize)> = Vec::new();
        for &unit in &rec.ours_lost {
            // A brigade's battalions are entered together ("Four battalions
            // of the First Egyptian Brigade"), as the rolls of the day did.
            let name = match identity(unit) {
                Some(i @ UnitIdentity::AngloEgyptianInfantry { brigade, .. })
                    if !i.is_friendlies() && scenario != Scenario::FallOfKhartoum =>
                {
                    format!("brigade {}", brigade.designation())
                }
                _ => super::prose_name(unit, scenario),
            };
            match counted.iter_mut().find(|(n, _, _)| *n == name) {
                Some((_, _, count)) => *count += 1,
                None => counted.push((name, unit, 1)),
            }
        }
        // Generals first, then the flotilla, then the rest as they fell.
        counted.sort_by_key(|(_, unit, _)| match identity(*unit) {
            Some(UnitIdentity::AngloEgyptianLeader(_)) => 0,
            Some(UnitIdentity::AngloEgyptianGunboat(_)) => 1,
            _ => 2,
        });
        counted
            .into_iter()
            .map(|(_, unit, count)| capitalize(&super::prose_count(unit, count, scenario)))
            .collect()
    };
    let pool = match scenario {
        Scenario::FallOfKhartoum => FEATURES_1885,
        Scenario::Campaign | Scenario::Historical => FEATURES_1898,
    };
    let features = pool
        .iter()
        .enumerate()
        .map(|(i, feature)| Article {
            head: feature.head.to_string(),
            decks: Vec::new(),
            dateline: None,
            paragraphs: feature
                .paragraphs
                .iter()
                .enumerate()
                .map(|(j, bank)| {
                    pick(bank, salt(&[game_salt, 200 + i as u64, j as u64])).to_string()
                })
                .collect(),
        })
        .collect();
    let adverts = {
        let first = (game_salt % ADVERTS.len() as u64) as usize;
        (0..2)
            .map(|k| &ADVERTS[(first + k) % ADVERTS.len()])
            .map(|a| Advert {
                head: a.head.to_string(),
                lines: a.lines.iter().map(|l| (*l).to_string()).collect(),
            })
            .collect()
    };

    FrontPage {
        masthead: "The London Gazette".to_string(),
        issue: issue.to_string(),
        date,
        price: "One Penny".to_string(),
        headline: template.headline.to_string(),
        lead,
        chronicle,
        telegrams: telegram_column,
        roll_of_honour,
        features,
        adverts,
    }
}

/// A count in a deck: "Thirty-four", "None".
fn deck_number(n: usize) -> String {
    if n == 0 {
        "None".into()
    } else {
        capitalize(&number_word(n))
    }
}

/// The lead article's head.
fn battle_head(scenario: Scenario) -> &'static str {
    match scenario {
        Scenario::FallOfKhartoum => "THE FALL OF KHARTOUM",
        Scenario::Campaign | Scenario::Historical => "THE BATTLE OF OMDURMAN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::turn_summary::TurnSummary;
    use crate::{GameTurnIndex, turn_track::GameTime};
    use omdurman_types::DayNight;

    fn finished(scenario: Scenario, result: GameResult, events: Vec<TurnEventRecord>) -> GameState {
        let mut state = GameState::new(scenario);
        state.turn_summaries.push(TurnSummary {
            turn: GameTurnIndex::new(4),
            time: GameTime::Noon,
            day_night: DayNight::Day,
            first_player: Player::AngloEgyptian,
            events,
        });
        state.game_over = true;
        state.game_result = Some(result);
        state
    }

    /// A shell on `at` and the unit it killed -- the way a howitzer kill is
    /// recorded (no fire-combat record of its own).
    fn shelled(unit: UnitId, at: HexCoord) -> Vec<TurnEventRecord> {
        vec![
            TurnEventRecord::HowitzerImpact {
                at,
                scattered: false,
                lost: false,
            },
            TurnEventRecord::UnitEliminated {
                unit,
                cause: crate::effects::ElimCause::Combat,
            },
        ]
    }

    #[test]
    fn the_lead_reports_what_the_record_holds() {
        let mut events = shelled(UnitId::Baggara_0_0, HexCoord::new(25, 7));
        events.extend(shelled(UnitId::Baggara_0_1, HexCoord::new(25, 7)));
        let state = finished(
            Scenario::Historical,
            GameResult::Historical {
                ae: crate::HistoricalVictoryLevel::Strategic,
                d: crate::HistoricalVictoryLevel::Draw,
            },
            events,
        );
        let page = front_page(&state, &[(1, "LINES HOLD FULL STOP".into())]);
        let text = page.lead.paragraphs.join(" ");
        // The fight, where and when, and the losses -- from the record only.
        assert!(text.contains("September 2"), "{text}");
        assert!(text.contains("Kerreri"), "{text}");
        assert!(!text.contains("no serious engagement"), "{text}");
        assert!(text.to_lowercase().contains("two"), "{text}");
        assert!(!text.contains('{'), "unfilled placeholder: {text}");
        assert_eq!(page.date, "Saturday, September 3, 1898");
        assert_eq!(page.telegrams.len(), 1);
        assert!(page.features.len() >= 3);
        // Deterministic.
        assert_eq!(
            page,
            front_page(&state, &[(1, "LINES HOLD FULL STOP".into())])
        );
    }

    #[test]
    fn the_lead_tells_of_the_zariba() {
        let state = finished(
            Scenario::Campaign,
            GameResult::Historical {
                ae: crate::HistoricalVictoryLevel::Strategic,
                d: crate::HistoricalVictoryLevel::Draw,
            },
            vec![TurnEventRecord::ZaribaBuilt {
                hexsides: 6,
                complete: true,
            }],
        );
        let text = front_page(&state, &[]).lead.paragraphs.join(" ");
        assert!(text.contains("zariba") && text.contains("Egeiga"), "{text}");
    }

    #[test]
    fn one_emir_is_singular_and_a_brigade_is_one_entry() {
        let mut events = shelled(UnitId::Sherif_0_0, HexCoord::new(25, 7));
        events.extend(shelled(UnitId::Kitchener_5_0, HexCoord::new(25, 7)));
        events.extend(shelled(UnitId::Kitchener_6_0, HexCoord::new(25, 7)));
        let state = finished(
            Scenario::Campaign,
            GameResult::Campaign(crate::CampaignVictoryLevel::Draw),
            events,
        );
        let page = front_page(&state, &[]);
        let text = page.lead.paragraphs.join(" ");
        assert!(!text.contains("The emirs Sherif are"), "{text}");
        assert_eq!(
            page.roll_of_honour,
            ["Two battalions of the First Egyptian Brigade"]
        );
    }

    /// Forts, guns and steamers are not "bands": the deck counts the tribes,
    /// and the works get a sentence of their own.
    #[test]
    fn forts_guns_and_steamers_are_named_as_such() {
        let mut events = shelled(UnitId::Baggara_0_0, HexCoord::new(25, 7));
        for unit in [
            UnitId::HadendowaForts_0_0,
            UnitId::Hadendowa_7_1,
            UnitId::KhalifaAbdullah_0_1,
            UnitId::KhalifaAbdullah_1_0,
        ] {
            events.extend(shelled(unit, HexCoord::new(25, 7)));
        }
        let state = finished(
            Scenario::Campaign,
            GameResult::Campaign(crate::CampaignVictoryLevel::Draw),
            events,
        );
        let page = front_page(&state, &[]);
        let text = page.lead.paragraphs.join(" ");
        assert!(
            text.contains("two forts, a battery and a steamer"),
            "{text}"
        );
        assert!(
            !text.contains("five bands") && !text.contains("Five bands"),
            "{text}"
        );
        assert!(
            page.lead.decks.iter().any(|d| d.contains("One")),
            "{:?}",
            page.lead.decks
        );
    }

    #[test]
    fn khartoum_reports_gordons_fate() {
        let mut state = finished(
            Scenario::FallOfKhartoum,
            GameResult::FoK(crate::FoKVictoryLevel::DervishTactical),
            vec![],
        );
        state.gordon_eliminated_turn = Some(GameTurnIndex::new(3));
        // Gordon's messages came by runner; the fall, by telegraph from Korti.
        let heads: Vec<String> = front_page(&state, &[(2, "A".into()), (3, "B".into())])
            .telegrams
            .into_iter()
            .map(|(head, _)| head)
            .collect();
        assert!(heads[0].ends_with("(by runner)"), "{heads:?}");
        assert!(heads[1].starts_with("Korti"), "{heads:?}");
        let page = front_page(&state, &[]);
        let text = page.lead.paragraphs.join(" ");
        assert!(text.contains("Gordon"), "{text}");
        assert!(page.date.contains("1885"));
        assert!(page.features.iter().any(|f| f.head == "THE DESERT COLUMN"));
    }

    #[test]
    fn weekdays_match_the_calendar() {
        // The battle was fought on Friday, September 2, 1898.
        assert_eq!(weekday(1898, 9, 2), "Friday");
        assert_eq!(weekday(1885, 2, 5), "Thursday");
    }
}

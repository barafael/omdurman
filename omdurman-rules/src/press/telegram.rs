//! The field telegram of a finished turn: three to five facts, from the
//! British command's side (the Sirdar's headquarters at Omdurman, or
//! GORDON's palace at Khartoum), most important first, keyed in the
//! telegraphese of the day.

use omdurman_types::{HexCoord, Player, Scenario};

use super::phrases::TELEGRAM as T;
use super::{Bank, and_list, field_force, number_word, pick, salt, whereabouts};
use crate::effects::{ElimCause, GameState};
use crate::turn_summary::{TurnEventRecord, TurnSummary};
use crate::{CombatResult, UnitId, UnitIdentity, VpSource};

/// Facts a telegram carries at most / at least (the turn's own events
/// first; the situation fills a thin turn up to the minimum).
const MOST: usize = 5;
const LEAST: usize = 4;

/// One fact of the turn: its weight (lower is more important), the bank it
/// is worded from and the placeholder values.
struct Fact {
    weight: u8,
    kind: u64,
    bank: Bank,
    values: Vec<(&'static str, String)>,
}

impl Fact {
    fn new(weight: u8, kind: u64, bank: Bank) -> Self {
        Self {
            weight,
            kind,
            bank,
            values: Vec::new(),
        }
    }

    fn with(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.values.push((key, value.into()));
        self
    }
}

/// A unit's name as the telegraph clerk keys it ("Gunboat Naser", "3E
/// First Btn", "Baggara").
fn name(id: UnitId) -> String {
    crate::unit_profiles::profile_for_unit(id)
        .map_or_else(|| "a unit".to_string(), |p| p.identity.short_label())
}

fn identity(id: UnitId) -> Option<UnitIdentity> {
    crate::unit_profiles::profile_for_unit(id).map(|p| p.identity)
}

/// The most frequent of `hexes`, if any (where most of something happened).
fn usual(hexes: &[HexCoord]) -> Option<HexCoord> {
    let mut counts: Vec<(HexCoord, usize)> = Vec::new();
    for &h in hexes {
        match counts.iter_mut().find(|(c, _)| *c == h) {
            Some((_, n)) => *n += 1,
            None => counts.push((h, 1)),
        }
    }
    counts.into_iter().max_by_key(|&(_, n)| n).map(|(h, _)| h)
}

/// Our lost units as the clerk keys a list: "3E First Btn and battery", or
/// "five units including 3E First Btn and battery".
fn wire_list(units: &[UnitId]) -> String {
    let names = distinct(units.iter().map(|&u| super::wire_name(u)), 2);
    if units.len() <= 2 {
        names.join(" and ")
    } else {
        format!(
            "{} units including {}",
            number_word(units.len()),
            names.join(" and ")
        )
    }
}

/// Distinct names in first-seen order, at most `cap`.
fn distinct(names: impl IntoIterator<Item = String>, cap: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for n in names {
        if !out.contains(&n) {
            out.push(n);
        }
    }
    out.truncate(cap);
    out
}

/// The facts `summary`'s turn itself offers (no situation report), most
/// important first.
fn event_facts(state: &GameState, summary: &TurnSummary) -> Vec<Fact> {
    let scenario = state.scenario;
    let place = |hex: Option<HexCoord>| hex.map(|h| whereabouts(scenario, h)).unwrap_or_default();
    // Where each unit died (see `super::deaths`).
    let died_at = super::deaths(summary);
    let death_place = |id: UnitId| died_at.iter().find(|(u, _)| *u == id).and_then(|(_, h)| *h);

    let mut facts: Vec<Fact> = Vec::new();
    let mut our_lost: Vec<UnitId> = Vec::new();
    let mut their_lost: Vec<UnitId> = Vec::new();
    let mut melee_hexes: Vec<HexCoord> = Vec::new();
    let mut shaken = 0usize;
    let mut our_arrivals: Vec<UnitId> = Vec::new();
    let mut their_arrivals: Vec<(UnitId, HexCoord)> = Vec::new();
    // (breached hexes, attempts) by our guns / the enemy's.
    let mut breaches: [(Vec<HexCoord>, usize); 2] = Default::default();
    let mut shells: Vec<HexCoord> = Vec::new();
    let mut retired = false;
    let mut deserted = 0usize;
    for event in &summary.events {
        match event {
            TurnEventRecord::UnitEliminated { unit, cause } => {
                let at = death_place(*unit);
                match identity(*unit) {
                    Some(UnitIdentity::AngloEgyptianLeader(leader)) => {
                        if *cause == ElimCause::GordonAtPalace
                            || leader == crate::BritishLeader::Gordon
                        {
                            facts.push(Fact::new(0, 1, T.gordon_killed));
                        } else {
                            facts.push(
                                Fact::new(0, 2, T.leader_lost)
                                    .with("leader", leader.to_string())
                                    .with("place", place(at)),
                            );
                        }
                    }
                    Some(UnitIdentity::DervishLeader(crate::DervishLeader::KhalifaAbdullah)) => {
                        facts.push(Fact::new(0, 3, T.khalifa_killed).with("place", place(at)));
                    }
                    Some(UnitIdentity::DervishLeader(leader)) => facts.push(
                        Fact::new(1, 4, T.emir_killed)
                            .with("leader", leader.to_string())
                            .with("place", place(at)),
                    ),
                    Some(id) if id.owner() == Player::AngloEgyptian => {
                        if matches!(id, UnitIdentity::AngloEgyptianGunboat(_)) {
                            facts.push(
                                Fact::new(1, 5, T.gunboat_sunk)
                                    .with("units", super::wire_name(*unit)),
                            );
                        } else {
                            our_lost.push(*unit);
                        }
                    }
                    Some(_) => their_lost.push(*unit),
                    None => {}
                }
            }
            TurnEventRecord::VpScored {
                source: VpSource::MahdisTombTaken,
                ..
            } => facts.push(Fact::new(0, 6, T.tomb_taken)),
            TurnEventRecord::WallBreach {
                attacker,
                hexside,
                breached,
                ..
            } => {
                let side = &mut breaches[usize::from(*attacker == Player::Dervish)];
                side.1 += 1;
                if *breached {
                    side.0.push(hexside.a);
                }
            }
            TurnEventRecord::MeleeCombat { hex, .. } => melee_hexes.push(*hex),
            TurnEventRecord::FireCombat {
                result: CombatResult::Disrupt,
                attacker: Player::AngloEgyptian,
                ..
            } => shaken += 1,
            TurnEventRecord::Reinforcements { units, player, at } => match player {
                Player::AngloEgyptian => our_arrivals.extend(units),
                Player::Dervish => their_arrivals.extend(units.iter().map(|&u| (u, *at))),
            },
            TurnEventRecord::HowitzerImpact { at, .. } => shells.push(*at),
            TurnEventRecord::Retreat { unit, .. } => {
                retired |= identity(*unit).is_some_and(|i| i.owner() == Player::Dervish);
            }
            TurnEventRecord::Desertion { units, .. } => deserted += units.len(),
            _ => {}
        }
    }

    for (side, (hexes, attempts)) in breaches.iter().enumerate() {
        let (done, failed) = if side == 0 {
            (T.we_breached, T.we_failed_breach)
        } else {
            (T.enemy_breached, T.enemy_failed_breach)
        };
        if let Some(at) = usual(hexes) {
            facts.push(Fact::new(1, 7, done).with("place", place(Some(at))));
        } else if *attempts > 0 {
            facts.push(Fact::new(6, 8, failed));
        }
    }
    if !melee_hexes.is_empty() {
        facts.push(Fact::new(2, 9, T.melee).with("place", place(usual(&melee_hexes))));
    }
    if !our_lost.is_empty() {
        let at: Vec<HexCoord> = our_lost.iter().filter_map(|&u| death_place(u)).collect();
        facts.push(
            Fact::new(2, 10, T.our_losses)
                .with("units", wire_list(&our_lost))
                .with("place", place(usual(&at))),
        );
    }
    if !their_lost.is_empty() {
        let tribes = distinct(their_lost.iter().map(|&u| name(u)), 2);
        let at: Vec<HexCoord> = their_lost.iter().filter_map(|&u| death_place(u)).collect();
        facts.push(
            Fact::new(3, 11, T.enemy_losses)
                .with("n", number_word(their_lost.len()))
                .with("tribes", and_list(&tribes))
                .with("place", place(usual(&at))),
        );
    }
    if deserted > 0 {
        facts.push(Fact::new(3, 12, T.desertion).with("n", number_word(deserted)));
    }
    if !our_arrivals.is_empty() {
        let kinds = distinct(
            our_arrivals.iter().filter_map(|&u| {
                identity(u).map(|i| match i {
                    UnitIdentity::AngloEgyptianGunboat(_) => "gunboats".to_string(),
                    UnitIdentity::AngloEgyptianInfantry { brigade, .. } if i.is_friendlies() => {
                        let _ = brigade;
                        "Friendlies".to_string()
                    }
                    UnitIdentity::AngloEgyptianInfantry { brigade, .. } => {
                        format!("{brigade} brigade")
                    }
                    UnitIdentity::AngloEgyptianLeader(l) => format!("General {l}"),
                    UnitIdentity::AngloEgyptianCavalry => "cavalry".into(),
                    UnitIdentity::AngloEgyptianCamelCorps => "Camel Corps".into(),
                    UnitIdentity::AngloEgyptianArtillery => "artillery".into(),
                    UnitIdentity::AngloEgyptianMaxim => "Maxims".into(),
                    UnitIdentity::RoyalEngineers => "Engineers".into(),
                    other => other.short_label(),
                })
            }),
            3,
        );
        facts.push(
            Fact::new(4, 13, T.our_arrivals)
                .with("n", number_word(our_arrivals.len()))
                .with("units", and_list(&kinds)),
        );
    }
    if !their_arrivals.is_empty() {
        // The tribes that came up (their emirs ride with them).
        let tribes = distinct(
            their_arrivals
                .iter()
                .filter(|&&(u, _)| matches!(identity(u), Some(UnitIdentity::DervishTribal { .. })))
                .map(|&(u, _)| name(u)),
            2,
        );
        let at: Vec<HexCoord> = their_arrivals.iter().map(|&(_, h)| h).collect();
        facts.push(
            Fact::new(4, 14, T.enemy_arrivals)
                .with("n", number_word(their_arrivals.len()))
                .with("tribes", and_list(&tribes))
                .with("place", place(usual(&at))),
        );
    }
    if shaken > 0 {
        facts.push(Fact::new(5, 15, T.enemy_shaken).with("n", number_word(shaken)));
    }
    if !shells.is_empty() {
        facts.push(Fact::new(5, 16, T.shelling).with("place", place(usual(&shells))));
    }
    if retired {
        facts.push(Fact::new(5, 17, T.enemy_retired));
    }

    facts.sort_by_key(|f| f.weight);
    facts
}

/// The telegram of `summary`'s turn, keyed in telegraphese. `state` is the
/// game as the turn left it: it places the armies when the turn itself
/// offers too few facts.
pub fn telegram(state: &GameState, summary: &TurnSummary) -> String {
    let scenario = state.scenario;
    let place = |hex: Option<HexCoord>| hex.map(|h| whereabouts(scenario, h)).unwrap_or_default();
    let mut facts = event_facts(state, summary);
    // The situation as the turn left it, to fill a thin telegram.
    let ours = field_force(state, Player::AngloEgyptian);
    let theirs = field_force(state, Player::Dervish);
    let mut situation: Vec<Fact> = Vec::new();
    if scenario == Scenario::FallOfKhartoum && state.gordon_eliminated_turn.is_none() {
        situation.push(Fact::new(7, 20, T.gordon_holds));
    }
    let nearest = ours
        .iter()
        .flat_map(|a| theirs.iter().map(move |b| a.distance(*b)))
        .min();
    match nearest {
        Some(d) if d <= 2 => situation.push(Fact::new(7, 21, T.in_contact)),
        Some(d) => {
            let miles = ((d + 2) / 4).max(1) as usize;
            situation.push(Fact::new(8, 22, T.enemy_distance).with("n", number_word(miles)));
        }
        None => {}
    }
    if let Some(at) = super::centre_of(&theirs) {
        situation.push(Fact::new(8, 23, T.enemy_body).with("place", place(Some(at))));
    }
    if scenario == Scenario::Campaign {
        match crate::effects::mahdis_tomb_controller(state) {
            Some(Player::Dervish) => situation.push(Fact::new(8, 24, T.tomb_enemy)),
            Some(Player::AngloEgyptian) => situation.push(Fact::new(8, 24, T.tomb_ours)),
            None => {}
        }
        // Only the Campaign keeps victory points (§9.14; §9.24 counts units,
        // §9.35 Dervish losses).
        let a = state.victory.total_for(Player::AngloEgyptian).value();
        let d = state.victory.total_for(Player::Dervish).value();
        if a != 0 || d != 0 {
            situation.push(
                Fact::new(9, 25, T.score)
                    .with("ours", a.to_string())
                    .with("theirs", d.to_string()),
            );
        }
    }
    if let Some(at) = super::centre_of(&ours).filter(|_| scenario != Scenario::FallOfKhartoum) {
        situation.push(Fact::new(9, 26, T.our_position).with("place", place(Some(at))));
    }
    if summary.day_night == omdurman_types::DayNight::Day
        && crate::turn_track::scenario_turn(
            scenario,
            crate::GameTurnIndex::new(summary.turn.value() + 1),
        )
        .is_some_and(|next| next.day_night == omdurman_types::DayNight::Night)
    {
        situation.push(Fact::new(9, 27, T.night));
    }

    facts.truncate(MOST);
    for fact in situation {
        if facts.len() >= LEAST {
            break;
        }
        facts.push(fact);
    }
    if facts.is_empty() {
        facts.push(Fact::new(9, 28, T.quiet));
    }

    telegraphese(&word(&facts, summary, scenario).join(". "))
}

/// The facts as sentences (ordinary case, before telegraphese).
fn word(facts: &[Fact], summary: &TurnSummary, scenario: Scenario) -> Vec<String> {
    let turn = u64::from(summary.turn.value());
    facts
        .iter()
        .map(|fact| {
            let wording = pick(fact.bank, salt(&[turn, fact.kind, scenario as u64]));
            let values: Vec<(&str, &str)> =
                fact.values.iter().map(|(k, v)| (*k, v.as_str())).collect();
            super::fill(wording, &values)
        })
        .collect()
}

/// What happened in `summary`'s turn as plain sentences for the
/// newspaper's chronicle -- the telegram's own facts (not the situation
/// report), at most four.
pub fn chronicle(state: &GameState, summary: &TurnSummary) -> Vec<String> {
    let mut facts = event_facts(state, summary);
    facts.truncate(4);
    word(&facts, summary, state.scenario)
}

/// `text` keyed as a telegram of the day: all capitals, no punctuation,
/// every sentence ended by STOP and the message by FULL STOP. Idempotent;
/// the overlay also keys records filed before the style.
pub fn telegraphese(text: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut current = String::new();
    let end_sentence = |words: &mut Vec<String>| {
        if words.last().is_some_and(|w| w != "STOP") {
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
    // "FULL STOP" inside the text is a sentence end; doubled STOPs collapse.
    let mut out: Vec<String> = Vec::new();
    for word in words {
        if word == "STOP" && out.last().is_some_and(|w| w == "FULL") {
            out.pop();
            if out.last().is_some_and(|w| w != "STOP") {
                out.push("STOP".into());
            }
            continue;
        }
        if word == "STOP" && out.last().is_none_or(|w| w == "STOP") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GameTurnIndex;
    use crate::board::BoardInfo;
    use crate::turn_track::GameTime;
    use omdurman_types::DayNight;

    fn campaign() -> GameState {
        GameState::with_board(
            Scenario::Campaign,
            BoardInfo::from_map_data(&crate::board_data::campaign_map_data()),
        )
    }

    fn summary(events: Vec<TurnEventRecord>) -> TurnSummary {
        TurnSummary {
            turn: GameTurnIndex::new(2),
            time: GameTime::EightAM,
            day_night: DayNight::Day,
            first_player: Player::AngloEgyptian,
            events,
        }
    }

    fn place(state: &mut GameState, id: UnitId, at: HexCoord) {
        state.units.push(crate::UnitPlacement {
            id,
            position: at,
            profile: crate::unit_profiles::profile_for_unit(id).expect("a counter"),
            state: Default::default(),
        });
    }

    fn facts(text: &str) -> usize {
        text.matches("STOP").count()
    }

    #[test]
    fn telegraphese_keys_stops_and_capitals() {
        assert_eq!(
            telegraphese("The enemy advanced on Kerreri. Gunboats engaged, heavily!"),
            "THE ENEMY ADVANCED ON KERRERI STOP GUNBOATS ENGAGED HEAVILY FULL STOP"
        );
        let keyed = "DERVISH MASSING STOP LINES HOLD FULL STOP";
        assert_eq!(telegraphese(keyed), keyed);
        assert_eq!(
            telegraphese("Lines hold STOP. Full stop."),
            "LINES HOLD FULL STOP"
        );
        assert_eq!(telegraphese(""), "ALL QUIET STOP LINES HOLD FULL STOP");
        assert_eq!(
            telegraphese("Mahdi's Tomb held; 3E First Btn lost"),
            "MAHDI'S TOMB HELD STOP 3E FIRST BTN LOST FULL STOP"
        );
    }

    #[test]
    fn a_quiet_turn_still_carries_three_facts() {
        // Nothing happened, but both armies stand on the map: the telegram
        // places them rather than saying nothing.
        let mut state = campaign();
        place(&mut state, UnitId::EgyptianArmy_0_1, HexCoord::new(24, 3));
        place(&mut state, UnitId::Baggara_0_0, HexCoord::new(20, 20));
        place(&mut state, UnitId::Taiasha_0_0, HexCoord::new(31, 38));
        let text = telegram(&state, &summary(vec![]));
        assert!((3..=5).contains(&facts(&text)), "{text}");
        assert!(text.ends_with("FULL STOP"), "{text}");
        assert!(!text.contains('('), "no coordinates: {text}");
    }

    #[test]
    fn a_busy_turn_keeps_the_five_most_important() {
        let mut state = campaign();
        place(&mut state, UnitId::EgyptianArmy_0_1, HexCoord::new(24, 3));
        place(&mut state, UnitId::Baggara_0_0, HexCoord::new(24, 5));
        let killed: Vec<UnitId> = vec![UnitId::Baggara_1_0, UnitId::Baggara_1_1];
        let events = vec![
            TurnEventRecord::FireCombat {
                attacker: Player::AngloEgyptian,
                firers: vec![UnitId::EgyptianArmy_0_1],
                target: HexCoord::new(24, 5),
                roll: crate::DieRoll::try_from(9).unwrap(),
                modifiers: vec![],
                total_modifier: 1,
                result: CombatResult::Eliminate(2),
                kind: crate::FireKind::Direct,
                eliminated: killed.clone(),
            },
            TurnEventRecord::UnitEliminated {
                unit: killed[0],
                cause: ElimCause::Combat,
            },
            TurnEventRecord::UnitEliminated {
                unit: killed[1],
                cause: ElimCause::Combat,
            },
            TurnEventRecord::UnitEliminated {
                unit: UnitId::Kitchener_1_0,
                cause: ElimCause::OrphanLeader,
            },
            TurnEventRecord::Reinforcements {
                units: vec![UnitId::BritishBoats_3_0],
                player: Player::AngloEgyptian,
                at: HexCoord::new(28, 0),
            },
        ];
        let text = telegram(&state, &summary(events.clone()));
        assert!((3..=5).contains(&facts(&text)), "{text}");
        // The leader's loss leads, the enemy's losses follow.
        assert!(text.contains("GATACRE"), "{text}");
        assert!(text.contains("TWO") || text.contains("BAGGARA"), "{text}");
        // Deterministic: the same turn reads the same.
        assert_eq!(text, telegram(&state, &summary(events)));
    }

    /// Only the Campaign keeps victory points: the Historical scenario
    /// (§9.24, units eliminated) and FALL OF KHARTOUM never report points.
    #[traceability_macro::rulebook("§9.24")]
    #[test]
    fn only_the_campaign_reports_victory_points() {
        for scenario in [
            Scenario::Campaign,
            Scenario::Historical,
            Scenario::FallOfKhartoum,
        ] {
            let mut state = GameState::new(scenario);
            state.victory.events.push(crate::VpEvent {
                turn: GameTurnIndex::new(1),
                source: VpSource::DervishUnitEliminated,
            });
            let text = telegram(&state, &summary(vec![]));
            assert_eq!(
                text.contains("POINTS") || text.contains("SCORE"),
                scenario == Scenario::Campaign,
                "{scenario:?}: {text}"
            );
        }
    }
}

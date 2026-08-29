use serde::{Deserialize, Serialize};

use crate::GameTurnIndex;
use omdurman_types::{DayNight, Scenario};

/// Wall-clock time for a turn on the Turn Record Track (rulebook §9.12, §9.22).
///
/// The battle spans Sept 1 6:00 am through Sept 3 8:00 am; every turn
/// starts at one of these twelve times.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameTime {
    SixAM,
    EightAM,
    TenAM,
    Noon,
    TwoPM,
    FourPM,
    SixPM,
    EightPM,
    TenPM,
    Midnight,
    TwoAM,
    FourAM,
}

impl std::fmt::Display for GameTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameTime::SixAM => write!(f, "6:00 am"),
            GameTime::EightAM => write!(f, "8:00 am"),
            GameTime::TenAM => write!(f, "10:00 am"),
            GameTime::Noon => write!(f, "12:00 pm"),
            GameTime::TwoPM => write!(f, "2:00 pm"),
            GameTime::FourPM => write!(f, "4:00 pm"),
            GameTime::SixPM => write!(f, "6:00 pm"),
            GameTime::EightPM => write!(f, "8:00 pm"),
            GameTime::TenPM => write!(f, "10:00 pm"),
            GameTime::Midnight => write!(f, "12:00 am"),
            GameTime::TwoAM => write!(f, "2:00 am"),
            GameTime::FourAM => write!(f, "4:00 am"),
        }
    }
}

/// A single entry on the Turn Record Track (rulebook §9.12, §9.22).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct TurnEntry {
    /// 1-based turn number.
    pub turn: u8,
    /// Wall-clock time.
    pub time: GameTime,
    /// Day or night.
    pub day_night: DayNight,
    /// Any special event on this turn.
    pub event: TurnEvent,
}

/// Special events that occur on specific turns (rulebook §8.2, §9.112, §9.113).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TurnEvent {
    None,
    /// Dervish desertion roll (§8.2) -- occurs on the first night turn.
    DervishDesertion,
    /// Dervish reinforcements are available.
    DervishReinforcements,
    /// Anglo-Egyptian reinforcements are available.
    AngloEgyptianReinforcements,
}

/// Compact constructor for a track entry (keeps the literal table readable).
const fn entry(turn: u8, time: GameTime, day_night: DayNight, event: TurnEvent) -> TurnEntry {
    TurnEntry {
        turn,
        time,
        day_night,
        event,
    }
}

// Short names so the track tables below read as compact rows.
use DayNight::{Day, Night};
use GameTime::{EightAM, EightPM, FourPM, Midnight, Noon, SixAM, SixPM, TenAM, TenPM, TwoPM};

/// Campaign Game Turn Record Track (§9.12 -- 22 turns, Sept 1 6:00 am through
/// Sept 3 8:00 am).
///
/// Transcribed from the printed Turn Record Track (`CampaignTiming.jpg`), which
/// is a boustrophedon (snake) layout: row 1 left->right (turns 1-9), row 2
/// right->left (turns 10-18), row 3 left->right (turns 19-22). The 50-hour span
/// is not a uniform 2-hour cadence: each of the two nights is represented by a
/// pair of NIGHT turns (the printed cells carry no clock time, only "NIGHT"),
/// while day turns advance 6 am -> 8 pm in 2-hour steps. The first night turn
/// is turn 9, which carries the once-per-game Dervish Desertion Roll (§8.2) --
/// the printed track prints "Dervish Desertion Roll / NIGHT" on that cell.
pub const CAMPAIGN_TURN_TRACK: [TurnEntry; 22] = [
    // Row 1, left->right: Sept 1, 6 am -> 8 pm, then the first NIGHT.
    entry(1, SixAM, Day, TurnEvent::None),
    entry(2, EightAM, Day, TurnEvent::None),
    entry(3, TenAM, Day, TurnEvent::None),
    entry(4, Noon, Day, TurnEvent::None),
    entry(5, TwoPM, Day, TurnEvent::None),
    entry(6, FourPM, Day, TurnEvent::None),
    entry(7, SixPM, Day, TurnEvent::None),
    entry(8, EightPM, Day, TurnEvent::None),
    // First night turn (§8.2 desertion roll happens here).
    entry(9, TenPM, Night, TurnEvent::DervishDesertion),
    // Row 2, right->left: the second cell of the Sept 1->2 night, then Sept 2
    // 6 am -> 8 pm.
    entry(10, Midnight, Night, TurnEvent::None),
    entry(11, SixAM, Day, TurnEvent::None),
    entry(12, EightAM, Day, TurnEvent::None),
    entry(13, TenAM, Day, TurnEvent::None),
    entry(14, Noon, Day, TurnEvent::None),
    entry(15, TwoPM, Day, TurnEvent::None),
    entry(16, FourPM, Day, TurnEvent::None),
    entry(17, SixPM, Day, TurnEvent::None),
    entry(18, EightPM, Day, TurnEvent::None),
    // Row 3, left->right: the Sept 2->3 night (two NIGHT cells), then Sept 3
    // 6 am and 8 am.
    entry(19, TenPM, Night, TurnEvent::None),
    entry(20, Midnight, Night, TurnEvent::None),
    entry(21, SixAM, Day, TurnEvent::None),
    entry(22, EightAM, Day, TurnEvent::None),
];

/// Get the turn entry for a given 1-based turn index (campaign game).
pub fn campaign_turn(turn: GameTurnIndex) -> Option<&'static TurnEntry> {
    CAMPAIGN_TURN_TRACK.get((turn.value() as usize).saturating_sub(1))
}

/// Historical scenario track (§9.22 -- 4 turns, Sept 2 6:00 am -> 12:00 pm).
pub const HISTORICAL_TURN_TRACK: [TurnEntry; 4] = [
    TurnEntry {
        turn: 1,
        time: GameTime::SixAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 2,
        time: GameTime::EightAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 3,
        time: GameTime::TenAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 4,
        time: GameTime::Noon,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
];

pub fn historical_turn(turn: GameTurnIndex) -> Option<&'static TurnEntry> {
    HISTORICAL_TURN_TRACK.get((turn.value() as usize).saturating_sub(1))
}

/// Fall of Khartoum Turn Record Track (§9.33, §9.341, §9.35).
///
/// The scenario has no printed wall-clock track: it is variable length and
/// "rarely lasts five turns" (§9.33), with victory checked by which turn GORDON
/// dies, up to "survives end of turn eight" (§9.35). Turn 1 is *always* a night
/// turn (§9.341); the assault begins in the pre-dawn hours, so the remaining
/// turns run through the following morning. The `time` values are illustrative
/// (the rulebook fixes none); only `day_night` is rule-bearing (night halves
/// Anglo-Egyptian movement and ranges and bars howitzer fire, §8.1).
pub const FALL_OF_KHARTOUM_TURN_TRACK: [TurnEntry; 8] = [
    TurnEntry {
        turn: 1,
        time: GameTime::TwoAM,
        day_night: DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 2,
        time: GameTime::FourAM,
        day_night: DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 3,
        time: GameTime::SixAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 4,
        time: GameTime::EightAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 5,
        time: GameTime::TenAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 6,
        time: GameTime::Noon,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 7,
        time: GameTime::TwoPM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 8,
        time: GameTime::FourPM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
];

pub fn fall_of_khartoum_turn(turn: GameTurnIndex) -> Option<&'static TurnEntry> {
    FALL_OF_KHARTOUM_TURN_TRACK.get((turn.value() as usize).saturating_sub(1))
}

/// The turn entry for a scenario's 1-based turn index, routing each scenario to
/// its own Turn Record Track (§9.12 campaign, §9.22 historical, §9.33/§9.341
/// Fall of Khartoum). `None` past the end of the scenario (game over).
pub fn scenario_turn(scenario: Scenario, turn: GameTurnIndex) -> Option<&'static TurnEntry> {
    match scenario {
        Scenario::Campaign => campaign_turn(turn),
        Scenario::Historical => historical_turn(turn),
        Scenario::FallOfKhartoum => fall_of_khartoum_turn(turn),
    }
}

/// Labels for the campaign turn-track cells on the printed mapsheet.
/// The track is a 9 × 3 grid with a snake layout. Each variant is one distinct
/// printed cell; `Blank` is for unused positions in the 9×3 grid.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnLabel {
    Blank,
    /// "SEPT. 1\n6:00 am" -- day header plus time (turn 1).
    Sept1DayStart,
    /// "SEPT. 2\nNIGHT" -- day header plus night (turn 10).
    Sept2Night,
    /// "SEPT. 3\nNIGHT" -- day header plus night (turn 20).
    Sept3Night,
    /// "6:00 am" (turns 11 and 21).
    SixAm,
    /// "8:00" (turns 2, 8, 12, 18, 22).
    EightAm,
    /// "10:00" (turns 3, 13).
    TenAm,
    /// "12:00" (turns 4, 14).
    Noon,
    /// "2:00 pm" (turns 5, 15).
    TwoPm,
    /// "4:00" (turns 6, 16).
    FourPm,
    /// "6:00" (turns 7, 17).
    SixPm,
    /// "8:00" (turns 8, 18).
    EightPm,
    /// "NIGHT" (turns 9, 19).
    Night,
}

/// One variant per cell of the 9×3 printed grid, in 1-based turn order. `Blank`
/// fills the unused trailing positions of row 2 (turns 23–27).
const CAMPAIGN_TURN_LABELS: [TurnLabel; 22] = [
    // Row 0  L→R: Sept 1
    TurnLabel::Sept1DayStart,
    TurnLabel::EightAm,
    TurnLabel::TenAm,
    TurnLabel::Noon,
    TurnLabel::TwoPm,
    TurnLabel::FourPm,
    TurnLabel::SixPm,
    TurnLabel::EightPm,
    TurnLabel::Night,
    // Row 1 R→L: Sept 2 (T10 rightmost, T18 leftmost)
    TurnLabel::Sept2Night,
    TurnLabel::SixAm,
    TurnLabel::EightAm,
    TurnLabel::TenAm,
    TurnLabel::Noon,
    TurnLabel::TwoPm,
    TurnLabel::FourPm,
    TurnLabel::SixPm,
    TurnLabel::EightPm,
    // Row 2 L→R: Sept 3
    TurnLabel::Night,
    TurnLabel::Sept3Night,
    TurnLabel::SixAm,
    TurnLabel::EightAm,
];

impl TurnLabel {
    /// Return the label for a 1-based campaign turn number (1..=22).
    pub fn from_turn(turn: u8) -> Option<Self> {
        CAMPAIGN_TURN_LABELS
            .get((turn as usize).checked_sub(1)?)
            .copied()
    }
}

impl std::fmt::Display for TurnLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blank => f.write_str(""),
            Self::Sept1DayStart => f.write_str("SEPT. 1\n6:00 am"),
            Self::Sept2Night => f.write_str("SEPT. 2\nNIGHT"),
            Self::Sept3Night => f.write_str("SEPT. 3\nNIGHT"),
            Self::SixAm => f.write_str("6:00 am"),
            Self::EightAm => f.write_str("8:00"),
            Self::TenAm => f.write_str("10:00"),
            Self::Noon => f.write_str("12:00"),
            Self::TwoPm => f.write_str("2:00 pm"),
            Self::FourPm => f.write_str("4:00"),
            Self::SixPm => f.write_str("6:00"),
            Self::EightPm => f.write_str("8:00"),
            Self::Night => f.write_str("NIGHT"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traceability_macro::rulebook;

    #[rulebook("§9.12")]
    #[test]
    fn campaign_track_22_turns() {
        assert!(campaign_turn(GameTurnIndex::new(1)).is_some());
        assert!(campaign_turn(GameTurnIndex::new(22)).is_some());
        assert!(campaign_turn(GameTurnIndex::new(23)).is_none());
    }

    #[rulebook("§8.2", "§9.12")]
    #[test]
    fn desertion_on_first_night() {
        // Per the printed track (CampaignTiming.jpg), turn 8 is the last Sept-1
        // day turn (8 pm) and turn 9 is the first NIGHT turn, which carries the
        // Dervish Desertion Roll (§8.2).
        let day = campaign_turn(GameTurnIndex::new(8)).unwrap();
        assert_eq!(day.day_night, DayNight::Day);
        assert_eq!(day.event, TurnEvent::None);

        let night = campaign_turn(GameTurnIndex::new(9)).unwrap();
        assert_eq!(night.day_night, DayNight::Night);
        assert_eq!(night.event, TurnEvent::DervishDesertion);
    }

    #[rulebook("§9.12")]
    #[test]
    fn campaign_track_label_and_day_night_agree() {
        // The rule-bearing CAMPAIGN_TURN_TRACK must agree with the printed
        // labels in TurnLabel::from_turn: every "NIGHT" cell is a Night turn
        // and every clock-time cell is a Day turn.
        for turn in 1u8..=22 {
            let entry = campaign_turn(GameTurnIndex(turn)).unwrap();
            let label = TurnLabel::from_turn(turn).unwrap();
            let text = label.to_string();
            let labelled_night = text.contains("NIGHT");
            assert_eq!(
                labelled_night,
                entry.day_night == DayNight::Night,
                "turn {turn}: label {text:?} disagrees with {:?}",
                entry.day_night
            );
        }
    }

    #[rulebook("§9.33", "§9.341")]
    #[test]
    fn fall_of_khartoum_turn_one_is_night() {
        // §9.341: turn 1 is always a night turn.
        let t = fall_of_khartoum_turn(GameTurnIndex::new(1)).unwrap();
        assert_eq!(t.day_night, DayNight::Night);
        // §9.33/§9.35: the scenario can run as far as turn 8.
        assert!(fall_of_khartoum_turn(GameTurnIndex::new(8)).is_some());
        assert!(fall_of_khartoum_turn(GameTurnIndex::new(9)).is_none());
    }

    #[rulebook("§9.22")]
    #[test]
    fn historical_turn_all_four_turns() {
        let t1 = historical_turn(GameTurnIndex::new(1)).unwrap();
        assert_eq!(t1.time, GameTime::SixAM);
        assert_eq!(t1.day_night, DayNight::Day);
        assert_eq!(t1.event, TurnEvent::None);

        let t2 = historical_turn(GameTurnIndex::new(2)).unwrap();
        assert_eq!(t2.time, GameTime::EightAM);

        let t3 = historical_turn(GameTurnIndex::new(3)).unwrap();
        assert_eq!(t3.time, GameTime::TenAM);

        let t4 = historical_turn(GameTurnIndex::new(4)).unwrap();
        assert_eq!(t4.time, GameTime::Noon);

        assert!(historical_turn(GameTurnIndex::new(5)).is_none());
    }

    #[rulebook("§4")]
    #[test]
    fn scenario_turn_dispatches_correctly() {
        let campaign = scenario_turn(Scenario::Campaign, GameTurnIndex::new(1)).unwrap();
        assert_eq!(campaign.time, GameTime::SixAM);

        let historical = scenario_turn(Scenario::Historical, GameTurnIndex::new(1)).unwrap();
        assert_eq!(historical.time, GameTime::SixAM);

        let fok = scenario_turn(Scenario::FallOfKhartoum, GameTurnIndex::new(1)).unwrap();
        assert_eq!(fok.day_night, DayNight::Night);
    }

    #[rulebook("§9.33")]
    #[test]
    fn fall_of_khartoum_turns_3_to_8_are_day() {
        for turn in 3u8..=8 {
            let t = fall_of_khartoum_turn(GameTurnIndex(turn)).unwrap();
            assert_eq!(
                t.day_night,
                DayNight::Day,
                "Fall of Khartoum turn {turn} should be Day"
            );
            assert_eq!(t.event, TurnEvent::None);
        }
    }

    #[rulebook("§9.12")]
    #[test]
    fn game_time_display_all_variants() {
        assert_eq!(GameTime::SixAM.to_string(), "6:00 am");
        assert_eq!(GameTime::EightAM.to_string(), "8:00 am");
        assert_eq!(GameTime::TenAM.to_string(), "10:00 am");
        assert_eq!(GameTime::Noon.to_string(), "12:00 pm");
        assert_eq!(GameTime::TwoPM.to_string(), "2:00 pm");
        assert_eq!(GameTime::FourPM.to_string(), "4:00 pm");
        assert_eq!(GameTime::SixPM.to_string(), "6:00 pm");
        assert_eq!(GameTime::EightPM.to_string(), "8:00 pm");
        assert_eq!(GameTime::TenPM.to_string(), "10:00 pm");
        assert_eq!(GameTime::Midnight.to_string(), "12:00 am");
        assert_eq!(GameTime::TwoAM.to_string(), "2:00 am");
        assert_eq!(GameTime::FourAM.to_string(), "4:00 am");
    }

    #[rulebook("§9.12")]
    #[test]
    fn turn_label_display() {
        let label = TurnLabel::from_turn(1).unwrap();
        assert_eq!(label.to_string(), "SEPT. 1\n6:00 am");
        assert_eq!(TurnLabel::Blank.to_string(), "");
    }

    #[rulebook("§9.12")]
    #[test]
    fn turn_label_out_of_range_is_none() {
        assert!(TurnLabel::from_turn(0).is_none());
        assert!(TurnLabel::from_turn(23).is_none());
    }
}

use crate::{DayNight, GameTurnIndex};

/// Wall-clock time for a turn on the Turn Record Track (rulebook §9.12, §9.22).
///
/// The battle spans Sept 1 6:00 am through Sept 3 8:00 am; every turn
/// starts at one of these twelve times.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
pub fn scenario_turn(scenario: crate::Scenario, turn: GameTurnIndex) -> Option<&'static TurnEntry> {
    match scenario {
        crate::Scenario::Campaign => campaign_turn(turn),
        crate::Scenario::Historical => historical_turn(turn),
        crate::Scenario::FallOfKhartoum => fall_of_khartoum_turn(turn),
    }
}

/// Labels for the campaign turn-track cells on the printed mapsheet.
/// The track is a 9 × 3 grid with a snake layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnLabel {
    /// A cell that has no printed label (unused position in the 9×3 grid).
    Blank,
    /// A cell with its exact printed text.
    Text(&'static str),
}

impl TurnLabel {
    /// Return the label for a 1-based campaign turn number (1..=22).
    pub fn from_turn(turn: u8) -> Option<Self> {
        Some(match turn {
            // Row 0  L→R: Sept 1
            1 => Self::Text("SEPT. 1\n6:00 am"),
            2 => Self::Text("8:00"),
            3 => Self::Text("10:00"),
            4 => Self::Text("12:00"),
            5 => Self::Text("2:00 pm"),
            6 => Self::Text("4:00"),
            7 => Self::Text("6:00"),
            8 => Self::Text("8:00"),
            9 => Self::Text("NIGHT"),
            // Row 1 R→L: Sept 2 (T10 rightmost, T18 leftmost)
            10 => Self::Text("SEPT. 2\nNIGHT"),
            11 => Self::Text("6:00 am"),
            12 => Self::Text("8:00"),
            13 => Self::Text("10:00"),
            14 => Self::Text("12:00"),
            15 => Self::Text("2:00 pm"),
            16 => Self::Text("4:00"),
            17 => Self::Text("6:00"),
            18 => Self::Text("8:00"),
            // Row 2 L→R: Sept 3
            19 => Self::Text("NIGHT"),
            20 => Self::Text("SEPT. 3\nNIGHT"),
            21 => Self::Text("6:00 am"),
            22 => Self::Text("8:00"),
            _ => return None,
        })
    }
}

impl std::fmt::Display for TurnLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(t) => write!(f, "{t}"),
            Self::Blank => write!(f, ""),
        }
    }
}

/// Given a [`CampaignTurnTrack`] and a 1-based turn number, return the centre
/// pixel `(x, y)` on the campaign-map image where the turn marker should sit.
/// The 9 × 3 grid is laid out as:
///
/// | row | direction | valid turns |
/// |-----|-----------|-------------|
/// | 0   | L→R       | 1–9         |
/// | 1   | R→L       | 10–18       |
/// | 2   | L→R       | 19–22       |
///
/// Rows 0 and 1 use all 9 columns; row 2 uses only columns 0–3.
pub fn turn_marker_pixel(track: &omdurman_types::CampaignTurnTrack, turn: u8) -> (f32, f32) {
    let cell_w = track.w / 9.0;
    let cell_h = track.h / 3.0;
    let idx = (turn - 1) as usize;
    let row = idx / 9;
    let col = idx % 9;
    let cx = match row {
        0 | 2 => (col as f32 + 0.5) * cell_w,       // L→R
        1 => (9.0_f32 - col as f32 - 0.5) * cell_w, // R→L
        _ => 0.0,
    };
    let cy = (row as f32 + 0.5) * cell_h;
    (track.x + cx, track.y + cy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_track_22_turns() {
        assert!(campaign_turn(GameTurnIndex(1)).is_some());
        assert!(campaign_turn(GameTurnIndex(22)).is_some());
        assert!(campaign_turn(GameTurnIndex(23)).is_none());
    }

    #[test]
    fn desertion_on_first_night() {
        // Per the printed track (CampaignTiming.jpg), turn 8 is the last Sept-1
        // day turn (8 pm) and turn 9 is the first NIGHT turn, which carries the
        // Dervish Desertion Roll (§8.2).
        let day = campaign_turn(GameTurnIndex(8)).unwrap();
        assert_eq!(day.day_night, DayNight::Day);
        assert_eq!(day.event, TurnEvent::None);

        let night = campaign_turn(GameTurnIndex(9)).unwrap();
        assert_eq!(night.day_night, DayNight::Night);
        assert_eq!(night.event, TurnEvent::DervishDesertion);
    }

    #[test]
    fn campaign_track_label_and_day_night_agree() {
        // The rule-bearing CAMPAIGN_TURN_TRACK must agree with the printed
        // labels in TurnLabel::from_turn: every "NIGHT" cell is a Night turn
        // and every clock-time cell is a Day turn.
        for turn in 1u8..=22 {
            let entry = campaign_turn(GameTurnIndex(turn)).unwrap();
            let label = TurnLabel::from_turn(turn).unwrap();
            let TurnLabel::Text(text) = label else {
                panic!("turn {turn} has no label");
            };
            let labelled_night = text.contains("NIGHT");
            assert_eq!(
                labelled_night,
                entry.day_night == DayNight::Night,
                "turn {turn}: label {text:?} disagrees with {:?}",
                entry.day_night
            );
        }
    }

    #[test]
    fn fall_of_khartoum_turn_one_is_night() {
        // §9.341: turn 1 is always a night turn.
        let t = fall_of_khartoum_turn(GameTurnIndex(1)).unwrap();
        assert_eq!(t.day_night, DayNight::Night);
        // §9.33/§9.35: the scenario can run as far as turn 8.
        assert!(fall_of_khartoum_turn(GameTurnIndex(8)).is_some());
        assert!(fall_of_khartoum_turn(GameTurnIndex(9)).is_none());
    }
}

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

/// Campaign Game Turn Record Track (§9.12 -- 22 turns, Sept 1 6:00 am
/// through Sept 3 8:00 am).
///
/// Turns 1-4 are day turns on Sept 1, then night turns alternate with
/// day turns on Sept 2-3 per the printed track.
const CAMPAIGN_TURN_TRACK: [TurnEntry; 22] = [
    //  Sept 1
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
    TurnEntry {
        turn: 5,
        time: GameTime::TwoPM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 6,
        time: GameTime::FourPM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 7,
        time: GameTime::SixPM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 8,
        time: GameTime::EightPM,
        day_night: DayNight::Night,
        event: TurnEvent::DervishDesertion,
    },
    TurnEntry {
        turn: 9,
        time: GameTime::TenPM,
        day_night: DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 10,
        time: GameTime::Midnight,
        day_night: DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 11,
        time: GameTime::TwoAM,
        day_night: DayNight::Night,
        event: TurnEvent::None,
    },
    //  Sept 2
    TurnEntry {
        turn: 12,
        time: GameTime::FourAM,
        day_night: DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 13,
        time: GameTime::SixAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 14,
        time: GameTime::EightAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 15,
        time: GameTime::TenAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 16,
        time: GameTime::Noon,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 17,
        time: GameTime::TwoPM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 18,
        time: GameTime::FourPM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 19,
        time: GameTime::SixPM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 20,
        time: GameTime::EightPM,
        day_night: DayNight::Night,
        event: TurnEvent::None,
    },
    TurnEntry {
        turn: 21,
        time: GameTime::SixAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
    //  Sept 3
    TurnEntry {
        turn: 22,
        time: GameTime::EightAM,
        day_night: DayNight::Day,
        event: TurnEvent::None,
    },
];

/// Get the turn entry for a given 1-based turn index (campaign game).
pub fn campaign_turn(turn: GameTurnIndex) -> Option<&'static TurnEntry> {
    CAMPAIGN_TURN_TRACK.get((turn.value() as usize).saturating_sub(1))
}

/// Historical scenario track (§9.22 -- 4 turns, Sept 2 6:00 am -> 12:00 pm).
const HISTORICAL_TURN_TRACK: [TurnEntry; 4] = [
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
pub fn turn_marker_pixel(
    track: &omdurman_types::CampaignTurnTrack,
    turn: u8,
) -> (f32, f32) {
    let cell_w = track.w / 9.0;
    let cell_h = track.h / 3.0;
    let idx = (turn - 1) as usize;
    let row = idx / 9;
    let col = idx % 9;
    let cx = match row {
        0 | 2 => (col as f32 + 0.5) * cell_w,    // L→R
        1 => (9.0_f32 - col as f32 - 0.5) * cell_w,     // R→L
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
        let t = campaign_turn(GameTurnIndex(8)).unwrap();
        assert_eq!(t.day_night, DayNight::Night);
        assert_eq!(t.event, TurnEvent::DervishDesertion);
    }
}

use crate::{DayNight, GameTurnIndex};

/// Wall-clock time for a turn on the Turn Record Track.
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

/// A single entry on the Turn Record Track.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TurnEntry {
    /// 1‑based turn number.
    pub turn: u8,
    /// Wall-clock time.
    pub time: GameTime,
    /// Day or night.
    pub day_night: DayNight,
    /// Any special event on this turn.
    pub event: TurnEvent,
}

/// Special events that occur on specific turns.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TurnEvent {
    None,
    /// Dervish desertion roll (§8.2) — occurs on the first night turn.
    DervishDesertion,
    /// Dervish reinforcements are available.
    DervishReinforcements,
    /// Anglo-Egyptian reinforcements are available.
    AngloEgyptianReinforcements,
}

/// Campaign Game Turn Record Track (§9.12 — 22 turns, Sept 1 6:00 am
/// through Sept 3 8:00 am).
///
/// Turns 1–4 are day turns on Sept 1, then night turns alternate with
/// day turns on Sept 2–3 per the printed track.
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

/// Get the turn entry for a given 1‑based turn index (campaign game).
pub fn campaign_turn(turn: GameTurnIndex) -> Option<&'static TurnEntry> {
    CAMPAIGN_TURN_TRACK.get((turn.value() as usize).saturating_sub(1))
}

/// Historical scenario track (§9.22 — 4 turns, Sept 2 6:00 am → 12:00 pm).
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

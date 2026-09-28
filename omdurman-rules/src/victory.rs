//! Victory ledger, victory levels, and the game result.

use serde::{Deserialize, Serialize};

use crate::{GameTurnIndex, VictoryPoints};
use omdurman_types::Player;

// ---------------------------------------------------------------------------
// 15) Victory ledger
// ---------------------------------------------------------------------------

/// Every distinct VP source the rulebook enumerates (§9.14). Each variant
/// carries its point value as a method so the table cannot drift between
/// the manual and the engine.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum VpSource {
    // ----- Anglo-Egyptian player receives:
    /// 25 pts -- the Mahdi's Tomb taken from the Dervish: held at the
    /// conclusion of play by a British leader plus a non-"Friendlies"
    /// Anglo-Egyptian combat unit, both undisrupted (§9.14).
    MahdisTombTaken,
    /// 1 pt -- eliminating the Isa Zachneih unit (§9.14).
    IsaZachneihEliminated,
    /// 10 pts -- eliminating the Khalifa Abdullah (§9.14).
    KhalifaEliminated,
    /// 1 pt -- each Dervish unit eliminated (gunboats, artillery, other
    /// leaders included). Forts elimination is worth 0 pts (§9.14).
    DervishUnitEliminated,
    // ----- Dervish player receives:
    /// 25 pts -- the Mahdi's Tomb still Dervish-controlled at the conclusion
    /// of play: they control it from the start, so it is theirs unless the
    /// Anglo-Egyptian player takes it (§9.14).
    MahdisTombHeld,
    /// 1 pt -- each "Friendlies" unit eliminated on the east bank (§9.14).
    FriendliesEastBankEliminated,
    /// 3 pts -- each "Friendlies" unit eliminated on the west bank (§9.14).
    FriendliesWestBankEliminated,
    /// 10 pts -- each British leader eliminated (§9.14).
    BritishLeaderEliminated,
    /// 10 pts -- each British gunboat sunk (§9.14).
    BritishGunboatSunk,
    /// 3 pts -- each Anglo-Egyptian land unit eliminated (§9.14).
    AngloEgyptianLandUnitEliminated,
}

impl VpSource {
    /// VP awarded to `who_scores()` (rulebook §9.14).
    pub fn points(self) -> VictoryPoints {
        match self {
            VpSource::MahdisTombTaken | VpSource::MahdisTombHeld => VictoryPoints::new(25),
            VpSource::IsaZachneihEliminated => VictoryPoints::new(1),
            VpSource::KhalifaEliminated => VictoryPoints::new(10),
            VpSource::DervishUnitEliminated => VictoryPoints::new(1),
            VpSource::BritishLeaderEliminated => VictoryPoints::new(10),
            VpSource::BritishGunboatSunk => VictoryPoints::new(10),
            VpSource::FriendliesEastBankEliminated => VictoryPoints::new(1),
            VpSource::FriendliesWestBankEliminated => VictoryPoints::new(3),
            VpSource::AngloEgyptianLandUnitEliminated => VictoryPoints::new(3),
        }
    }

    /// Which player receives these victory points (rulebook §9.14).
    pub fn who_scores(self) -> Player {
        match self {
            VpSource::MahdisTombTaken
            | VpSource::IsaZachneihEliminated
            | VpSource::KhalifaEliminated
            | VpSource::DervishUnitEliminated => Player::AngloEgyptian,
            VpSource::MahdisTombHeld
            | VpSource::FriendliesEastBankEliminated
            | VpSource::FriendliesWestBankEliminated
            | VpSource::BritishLeaderEliminated
            | VpSource::BritishGunboatSunk
            | VpSource::AngloEgyptianLandUnitEliminated => Player::Dervish,
        }
    }
}

impl std::fmt::Display for VpSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VpSource::MahdisTombTaken => write!(f, "Mahdi's Tomb taken"),
            VpSource::MahdisTombHeld => write!(f, "Mahdi's Tomb held"),
            VpSource::IsaZachneihEliminated => write!(f, "Isa Zachneih eliminated"),
            VpSource::KhalifaEliminated => write!(f, "Khalifa eliminated"),
            VpSource::DervishUnitEliminated => write!(f, "Dervish unit eliminated"),
            VpSource::BritishLeaderEliminated => write!(f, "British leader eliminated"),
            VpSource::BritishGunboatSunk => write!(f, "British gunboat sunk"),
            VpSource::FriendliesEastBankEliminated => write!(f, "Friendlies lost (east bank)"),
            VpSource::FriendliesWestBankEliminated => write!(f, "Friendlies lost (west bank)"),
            VpSource::AngloEgyptianLandUnitEliminated => {
                write!(f, "Anglo-Egyptian unit eliminated")
            }
        }
    }
}

/// Cumulative victory ledger for one scenario (rulebook §9.14).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct VictoryLedger {
    pub events: Vec<VpEvent>,
}

/// A single victory-point scoring event (rulebook §9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct VpEvent {
    pub turn: GameTurnIndex,
    pub source: VpSource,
}

impl VictoryLedger {
    /// Total victory points earned by a given player (rulebook §9.14).
    pub fn total_for(&self, player: Player) -> VictoryPoints {
        VictoryPoints(
            self.events
                .iter()
                .filter(|e| e.source.who_scores() == player)
                .map(|e| e.source.points().0)
                .sum(),
        )
    }

    /// Net superiority: positive = Anglo-Egyptian ahead, negative = Dervish ahead
    /// (rulebook §9.14).
    pub fn superiority(&self) -> VictoryPoints {
        VictoryPoints(
            self.total_for(Player::AngloEgyptian).value() - self.total_for(Player::Dervish).value(),
        )
    }

    /// The number of *enemy units eliminated* by `player`, used by the
    /// Historical scenario's unit-count victory schedule (§9.24). Every
    /// elimination/sinking source records one event per unit; the Mahdi's Tomb
    /// sources are control, not eliminations, so they are excluded.
    pub fn units_eliminated_by(&self, player: Player) -> i16 {
        self.events
            .iter()
            .filter(|e| {
                e.source.who_scores() == player
                    && !matches!(
                        e.source,
                        VpSource::MahdisTombTaken | VpSource::MahdisTombHeld
                    )
            })
            .count() as i16
    }

    /// Both sides' Historical levels (§9.24), Anglo-Egyptian first, from
    /// the units each has eliminated.
    pub fn historical_levels(&self) -> (HistoricalVictoryLevel, HistoricalVictoryLevel) {
        (
            HistoricalVictoryLevel::for_anglo_egyptian(
                self.units_eliminated_by(Player::AngloEgyptian),
            ),
            HistoricalVictoryLevel::for_dervish(self.units_eliminated_by(Player::Dervish)),
        )
    }
}

/// Campaign-game victory levels (§9.14).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CampaignVictoryLevel {
    Draw,
    Marginal(Player),
    Tactical(Player),
    Decisive(Player),
}

impl CampaignVictoryLevel {
    /// Assign a level from the net superiority (§9.14).
    pub fn from_superiority(s: VictoryPoints) -> Self {
        let net = s.0;
        // Positive -> Anglo-Egyptian thresholds: 15/30/50
        // Negative -> Dervish thresholds: 10/20/30 (rulebook §9.14 table)
        if net >= 50 {
            CampaignVictoryLevel::Decisive(Player::AngloEgyptian)
        } else if net >= 30 {
            CampaignVictoryLevel::Tactical(Player::AngloEgyptian)
        } else if net >= 15 {
            CampaignVictoryLevel::Marginal(Player::AngloEgyptian)
        } else if net >= 1 {
            // 1-14 = Draw for the Anglo-Egyptian side
            CampaignVictoryLevel::Draw
        } else if net >= -9 {
            // 1-9 Dervish superiority = Draw
            CampaignVictoryLevel::Draw
        } else if net >= -19 {
            CampaignVictoryLevel::Marginal(Player::Dervish)
        } else if net >= -29 {
            CampaignVictoryLevel::Tactical(Player::Dervish)
        } else {
            CampaignVictoryLevel::Decisive(Player::Dervish)
        }
    }
}

/// Historical-scenario victory levels (§9.24). Numeric so subtraction works
/// per the rulebook example ("decisive worth 5 minus strategic worth 4 = 1,
/// draw").
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum HistoricalVictoryLevel {
    Draw = 1,
    Marginal = 2,
    Tactical = 3,
    Strategic = 4,
    Decisive = 5,
}

impl HistoricalVictoryLevel {
    /// The levels in order, Draw first.
    const LADDER: [Self; 5] = [
        Self::Draw,
        Self::Marginal,
        Self::Tactical,
        Self::Strategic,
        Self::Decisive,
    ];

    /// §9.24, left column: the Dervish units the Anglo-Egyptian player must
    /// eliminate for Marginal, Tactical, Strategic and Decisive.
    const AE_STEPS: [i16; 4] = [30, 45, 60, 100];

    /// §9.24, right column: the Anglo-Egyptian units the Dervish player must
    /// eliminate for each level above Draw.
    const DERVISH_STEPS: [i16; 4] = [5, 10, 15, 30];

    fn steps(for_player: Player) -> [i16; 4] {
        match for_player {
            Player::AngloEgyptian => Self::AE_STEPS,
            Player::Dervish => Self::DERVISH_STEPS,
        }
    }

    /// The level `for_player` reaches by eliminating `eliminated` enemy
    /// units (§9.24).
    pub fn reached(for_player: Player, eliminated: i16) -> Self {
        let passed = Self::steps(for_player)
            .iter()
            .filter(|step| eliminated >= **step)
            .count();
        Self::LADDER[passed]
    }

    /// Anglo-Egyptian level from the number of Dervish units eliminated
    /// (§9.24 left column): 0-29 draw, 30-44 marginal, 45-59 tactical,
    /// 60-99 strategic, 100+ decisive.
    pub fn for_anglo_egyptian(dervish_eliminated: i16) -> Self {
        Self::reached(Player::AngloEgyptian, dervish_eliminated)
    }

    /// Dervish level from the number of Anglo-Egyptian units eliminated
    /// (§9.24 right column): 0-4 draw, 5-9 marginal, 10-14 tactical,
    /// 15-29 strategic, 30+ decisive.
    pub fn for_dervish(anglo_egyptian_eliminated: i16) -> Self {
        Self::reached(Player::Dervish, anglo_egyptian_eliminated)
    }

    /// The net result (§9.24): "the lower value victory level is then
    /// subtracted from the higher level" -- the difference, read on the same
    /// scale (1 draw ... 4 strategic), goes to the side with the higher
    /// level. Equal levels are a draw. `None` for the winner means a draw.
    pub fn net(ae: Self, d: Self) -> (Option<Player>, Self) {
        let diff = ae as i16 - d as i16;
        let level = Self::LADDER[(diff.unsigned_abs() as usize).saturating_sub(1).min(4)];
        let winner = match (level, diff.signum()) {
            (Self::Draw, _) => None,
            (_, 1) => Some(Player::AngloEgyptian),
            _ => Some(Player::Dervish),
        };
        (winner, level)
    }

    /// The fewest enemy units the side must have eliminated for the next
    /// level up (§9.24), or `None` at Decisive.
    pub fn next_threshold(self, for_player: Player) -> Option<i16> {
        Self::steps(for_player).get(self as usize - 1).copied()
    }
}

/// Fall-of-Khartoum victory levels (§9.35). The base level is set by the turn
/// GORDON is eliminated; the Dervish player then loses victory levels for his
/// own losses. Modelled on a signed ladder (Dervish-favourable is more
/// negative) so the loss penalty is a simple shift toward the British end.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum FoKVictoryLevel {
    DervishDecisive = -3,
    DervishTactical = -2,
    DervishMarginal = -1,
    BritishMarginal = 1,
    BritishTactical = 2,
    BritishDecisive = 3,
}

impl FoKVictoryLevel {
    pub(crate) const LADDER: [FoKVictoryLevel; 6] = [
        FoKVictoryLevel::DervishDecisive,
        FoKVictoryLevel::DervishTactical,
        FoKVictoryLevel::DervishMarginal,
        FoKVictoryLevel::BritishMarginal,
        FoKVictoryLevel::BritishTactical,
        FoKVictoryLevel::BritishDecisive,
    ];

    /// Fallback index into [`Self::LADDER`] when a base level is somehow not
    /// found there. Centred on the Dervish-Marginal / British-Marginal boundary.
    pub(crate) const DEFAULT_LADDER_IDX: usize = 3;

    /// The base level from when GORDON died (§9.35): eliminated turn ≤4 Dervish
    /// decisive, turn 5 tactical, turn 6 marginal; if he survives, the British
    /// level depends on how long he held out -- turn 6 British marginal, turn 7
    /// tactical, turn 8 (or later) decisive.
    ///
    /// `gordon_died_turn` is `None` if GORDON was still alive at scenario end;
    /// `scenario_end_turn` is the 1-based turn on which the game ended (the
    /// scenario's max is 8 per the FoK turn track).
    fn base(gordon_died_turn: Option<u8>, scenario_end_turn: u8) -> Self {
        match gordon_died_turn {
            Some(t) if t <= 4 => FoKVictoryLevel::DervishDecisive,
            Some(5) => FoKVictoryLevel::DervishTactical,
            // GORDON dead turn 6+ is off the table's intent (the scenario ends
            // by turn 8); treat a turn-6-or-later death as the weakest Dervish
            // win.
            Some(_) => FoKVictoryLevel::DervishMarginal,
            // GORDON survived -- the British level grows with how long he held.
            // The ladder starts at turn 6; ending before that yields the floor
            // (BritishMarginal) as a best-effort result (§9.35 doesn't cover it).
            None => match scenario_end_turn {
                t if t >= 8 => FoKVictoryLevel::BritishDecisive,
                7 => FoKVictoryLevel::BritishTactical,
                _ => FoKVictoryLevel::BritishMarginal,
            },
        }
    }

    /// How many victory levels the Dervish player forfeits for his own losses
    /// (§9.35): 1 level at 16-23 units lost, 2 at 24-31, 3 at 32+.
    pub fn loss_penalty(dervish_lost: i16) -> i16 {
        match dervish_lost {
            n if n >= 32 => 3,
            n if n >= 24 => 2,
            n if n >= 16 => 1,
            _ => 0,
        }
    }

    /// The next Dervish-loss threshold at which an additional victory level is
    /// forfeited (§9.35), or `None` if already at the maximum (32+) penalty.
    /// Used by the FoK victory-progress panel to show "next penalty at N".
    pub fn next_loss_threshold(dervish_lost: i16) -> Option<i16> {
        match dervish_lost {
            n if n < 16 => Some(16),
            n if n < 24 => Some(24),
            n if n < 32 => Some(32),
            _ => None,
        }
    }

    /// Final level: the turn-based base shifted toward the British end of the
    /// ladder by the Dervish loss penalty (§9.35). Worked example from the
    /// rulebook: GORDON dies turn 5 (tactical) with 24 Dervish losses (−2
    /// levels) nets a British marginal.
    pub fn resolve(gordon_died_turn: Option<u8>, scenario_end_turn: u8, dervish_lost: i16) -> Self {
        let base = Self::base(gordon_died_turn, scenario_end_turn);
        let base_idx = Self::LADDER
            .iter()
            .position(|l| *l == base)
            .unwrap_or(Self::DEFAULT_LADDER_IDX) as i16;
        let shifted =
            (base_idx + Self::loss_penalty(dervish_lost)).clamp(0, Self::LADDER.len() as i16 - 1);
        Self::LADDER[shifted as usize]
    }
}

impl std::fmt::Display for FoKVictoryLevel {
    /// Human-readable label with a space, e.g. "Dervish Decisive",
    /// "British Marginal" (§9.35). Used by the FoK victory-progress panel.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FoKVictoryLevel::DervishDecisive => "Dervish Decisive",
            FoKVictoryLevel::DervishTactical => "Dervish Tactical",
            FoKVictoryLevel::DervishMarginal => "Dervish Marginal",
            FoKVictoryLevel::BritishMarginal => "British Marginal",
            FoKVictoryLevel::BritishTactical => "British Tactical",
            FoKVictoryLevel::BritishDecisive => "British Decisive",
        };
        f.write_str(s)
    }
}

/// The typed result of a finished game, preserving the scenario-specific
/// victory level (rulebook §9.14, §9.24, §9.35). Replaces the former
/// stringly-typed `game_result: Option<String>`.
///
/// The Historical scenario scores each side on its own unit-elimination
/// ladder (§9.24), so its result carries *both* levels rather than a single
/// net level -- the newspaper layer compares them to pick a template.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameResult {
    Campaign(CampaignVictoryLevel),
    Historical {
        ae: HistoricalVictoryLevel,
        d: HistoricalVictoryLevel,
    },
    FoK(FoKVictoryLevel),
}

impl GameResult {
    /// The result as players read it, for the game-over banner, the end-of-game
    /// stats line and the newspaper prompt: "Anglo-Egyptian Tactical Victory",
    /// "Draw", "British Decisive" (FoK), or for the Historical scenario both
    /// sides' levels and their difference (§9.24).
    pub fn display_key(self) -> String {
        match self {
            GameResult::Campaign(level) => match level {
                CampaignVictoryLevel::Draw => "Draw".to_string(),
                CampaignVictoryLevel::Marginal(p) => format!("{p} Marginal Victory"),
                CampaignVictoryLevel::Tactical(p) => format!("{p} Tactical Victory"),
                CampaignVictoryLevel::Decisive(p) => format!("{p} Decisive Victory"),
            },
            GameResult::Historical { ae, d } => match HistoricalVictoryLevel::net(ae, d) {
                (None, _) => format!("Draw (Anglo-Egyptian {ae:?} vs Dervish {d:?})"),
                (Some(p), level) => {
                    format!("{p} {level:?} Victory (Anglo-Egyptian {ae:?} vs Dervish {d:?})")
                }
            },
            GameResult::FoK(level) => level.to_string(),
        }
    }

    /// The newspaper's date line: FALL OF KHARTOUM is January 1885, the
    /// Omdurman scenarios September 1898.
    pub fn date_line(self) -> &'static str {
        match self {
            GameResult::FoK(_) => "January 1885",
            GameResult::Campaign(_) | GameResult::Historical { .. } => "September 1898",
        }
    }

    /// The battle the newspaper reports on.
    pub fn battle_name(self) -> &'static str {
        match self {
            GameResult::FoK(_) => "the fall of Khartoum",
            GameResult::Campaign(_) | GameResult::Historical { .. } => "the Battle of Omdurman",
        }
    }
}

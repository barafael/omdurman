use serde::{Deserialize, Serialize};

use crate::DemolitionTarget;
use crate::effects::ElimCause;
use crate::turn_track::GameTime;
use crate::{
    CombatResult, DayNight, DieRoll, FireKind, FireModifier, GameTurnIndex, HexCoord, Player,
    UnitId, VictoryPoints, VpSource,
};

/// A single structured event recorded during a game turn.
///
/// Accumulated by `apply_effect` arms into `GameState::turn_events` and
/// snapshotted into a [`TurnSummary`] when the game turn advances.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TurnEventRecord {
    /// A unit moved from one hex to another.
    Movement {
        unit: UnitId,
        from: HexCoord,
        to: HexCoord,
        cost: i16,
    },
    /// A direct or Maxim-second fire attack resolved.
    FireCombat {
        attacker: Player,
        firers: Vec<UnitId>,
        target: HexCoord,
        roll: DieRoll,
        modifiers: Vec<FireModifier>,
        total_modifier: i16,
        result: CombatResult,
        kind: FireKind,
        eliminated: Vec<UnitId>,
    },
    /// Melee combat resolved (simultaneous, two rolls).
    MeleeCombat {
        attacker: Player,
        defender: Player,
        hex: HexCoord,
        attacker_roll: DieRoll,
        defender_roll: DieRoll,
        attacker_result: CombatResult,
        defender_result: CombatResult,
        attacker_losses: Vec<UnitId>,
        defender_losses: Vec<UnitId>,
        mandatory_advance: Option<u8>,
    },
    /// A cavalry/camel unit retreated before melee resolution.
    Retreat {
        unit: UnitId,
        from: HexCoord,
        to: HexCoord,
    },
    /// A unit advanced into a hex vacated by combat.
    AdvanceAfterCombat {
        unit: UnitId,
        from: HexCoord,
        to: HexCoord,
    },
    /// Reinforcements were placed on the map.
    Reinforcements {
        units: Vec<UnitId>,
        player: Player,
        at: HexCoord,
    },
    /// A Royal Engineers demolition was attempted.
    Demolition {
        engineer: UnitId,
        target: DemolitionTarget,
        success: bool,
    },
    /// Dervish units deserted (campaign, first night turn).
    Desertion { units: Vec<UnitId>, roll: DieRoll },
    /// A unit was eliminated.
    UnitEliminated { unit: UnitId, cause: ElimCause },
    /// A unit was disrupted.
    UnitDisrupted { unit: UnitId },
    /// A unit recovered from disruption.
    UnitRecovered { unit: UnitId },
    /// A howitzer shell impacted at `at` (§6.64) — `scattered` when the
    /// impact roll moved the shell off the aimed hex.
    HowitzerImpact { at: HexCoord, scattered: bool },
    /// Victory points were scored.
    VpScored {
        source: VpSource,
        points: VictoryPoints,
        for_player: Player,
    },
}

/// A structured summary of one complete game turn (both players' turns).
///
/// Stored as an append-only list on [`GameState`](crate::effects::GameState).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TurnSummary {
    pub turn: GameTurnIndex,
    pub time: GameTime,
    pub day_night: DayNight,
    pub first_player: Player,
    pub events: Vec<TurnEventRecord>,
}

impl TurnEventRecord {
    /// Format this event as a terse line suitable for a military dispatch.
    pub fn format_for_dispatch(&self) -> String {
        match self {
            TurnEventRecord::Movement {
                unit,
                from,
                to,
                cost,
            } => {
                format!("{unit:?} advanced from {from:?} to {to:?} (cost {cost})")
            }
            TurnEventRecord::FireCombat {
                attacker,
                target,
                roll,
                result,
                eliminated,
                ..
            } => {
                let elim_str = if eliminated.is_empty() {
                    String::new()
                } else {
                    format!("; casualties: {:?}", eliminated)
                };
                format!(
                    "{attacker} fire at {target:?}: rolled {} -> {result:?}{elim_str}",
                    roll.value(),
                )
            }
            TurnEventRecord::MeleeCombat {
                attacker,
                defender,
                hex,
                attacker_result,
                defender_result,
                attacker_losses,
                defender_losses,
                ..
            } => {
                format!(
                    "Melee at {hex:?}: {attacker} {:?} / {defender} {:?} (losses: A {:?}, D {:?})",
                    attacker_result, defender_result, attacker_losses, defender_losses,
                )
            }
            TurnEventRecord::Retreat { unit, from, to } => {
                format!("{unit:?} retreated from {from:?} to {to:?}")
            }
            TurnEventRecord::AdvanceAfterCombat { unit, from, to } => {
                format!("{unit:?} advanced from {from:?} to {to:?}")
            }
            TurnEventRecord::Reinforcements { units, player, at } => {
                format!("{player} reinforcements ({units:?}) placed at {at:?}")
            }
            TurnEventRecord::Demolition {
                engineer,
                target,
                success,
            } => {
                let outcome = if *success { "succeeded" } else { "failed" };
                format!("Demolition by {engineer:?} on {target:?} {outcome}")
            }
            TurnEventRecord::Desertion { units, roll } => {
                format!(
                    "Dervish desertion (roll {}): {:?} removed",
                    roll.value(),
                    units
                )
            }
            TurnEventRecord::UnitEliminated { unit, cause } => {
                format!("{unit:?} eliminated ({cause})")
            }
            TurnEventRecord::UnitDisrupted { unit } => {
                format!("{unit:?} disrupted")
            }
            TurnEventRecord::UnitRecovered { unit } => {
                format!("{unit:?} recovered")
            }
            TurnEventRecord::HowitzerImpact { at, scattered } => {
                if *scattered {
                    format!("Howitzer shell scattered to {at:?} (§6.64)")
                } else {
                    format!("Howitzer shell on target at {at:?}")
                }
            }
            TurnEventRecord::VpScored {
                source,
                points,
                for_player,
            } => {
                format!("{for_player} scored {source:?} ({points:?})")
            }
        }
    }
}

impl TurnSummary {
    /// Format the full turn as a structured text block for LLM input.
    pub fn format_for_llm(&self) -> String {
        let mut out = format!(
            "=== Turn {} ({}, {:?}) ===\n",
            self.turn.0, self.time, self.day_night,
        );
        for event in &self.events {
            out.push_str(&format!("- {}\n", event.format_for_dispatch()));
        }
        out
    }
}

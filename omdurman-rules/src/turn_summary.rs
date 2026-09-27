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

/// A unit's player-facing name ("Mulazmin", "1B First Btn") from its counter
/// id -- the dispatch lines feed players and the flavour-text model, which
/// must never see internal ids like `MulazminII_5_1`.
fn unit_name(id: &UnitId) -> String {
    crate::unit_profiles::profile_for_unit(*id)
        .map_or_else(|| format!("{id:?}"), |p| p.identity.short_label())
}

/// Names of several units, comma-separated ("none" when empty).
fn unit_names(ids: &[UnitId]) -> String {
    if ids.is_empty() {
        return "none".into();
    }
    ids.iter().map(unit_name).collect::<Vec<_>>().join(", ")
}

/// A hex as players read it: "(13, 5)".
fn hex(h: &HexCoord) -> String {
    format!("({}, {})", h.q, h.r)
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
                format!(
                    "{} moved from {} to {} (cost {cost})",
                    unit_name(unit),
                    hex(from),
                    hex(to)
                )
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
                    format!("; casualties: {}", unit_names(eliminated))
                };
                format!(
                    "{attacker} fire at {}: rolled {} -> {result:?}{elim_str}",
                    hex(target),
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
                    "Melee at {}: {attacker} {:?} / {defender} {:?} (losses: {attacker} {}, {defender} {})",
                    self::hex(hex),
                    attacker_result,
                    defender_result,
                    unit_names(attacker_losses),
                    unit_names(defender_losses),
                )
            }
            TurnEventRecord::Retreat { unit, from, to } => {
                format!(
                    "{} retreated from {} to {}",
                    unit_name(unit),
                    hex(from),
                    hex(to)
                )
            }
            TurnEventRecord::AdvanceAfterCombat { unit, from, to } => {
                format!(
                    "{} advanced from {} to {}",
                    unit_name(unit),
                    hex(from),
                    hex(to)
                )
            }
            TurnEventRecord::Reinforcements { units, player, at } => {
                format!(
                    "{player} reinforcements ({}) placed at {}",
                    unit_names(units),
                    hex(at)
                )
            }
            TurnEventRecord::Demolition {
                engineer,
                target,
                success,
            } => {
                let outcome = if *success { "succeeded" } else { "failed" };
                format!(
                    "Demolition by {} on {target:?} {outcome}",
                    unit_name(engineer)
                )
            }
            TurnEventRecord::Desertion { units, roll } => {
                format!(
                    "Dervish desertion (roll {}): {} removed",
                    roll.value(),
                    unit_names(units)
                )
            }
            TurnEventRecord::UnitEliminated { unit, cause } => {
                format!("{} eliminated ({cause})", unit_name(unit))
            }
            TurnEventRecord::UnitDisrupted { unit } => {
                format!("{} disrupted", unit_name(unit))
            }
            TurnEventRecord::UnitRecovered { unit } => {
                format!("{} recovered", unit_name(unit))
            }
            TurnEventRecord::HowitzerImpact { at, scattered } => {
                if *scattered {
                    format!("Howitzer shell scattered to {} (§6.64)", hex(at))
                } else {
                    format!("Howitzer shell on target at {}", hex(at))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Dispatch lines (shown to players and fed to the flavour-text model)
    /// name units and hexes the way the board does, never by internal id.
    #[test]
    fn dispatch_lines_use_player_facing_names() {
        let line = TurnEventRecord::UnitEliminated {
            unit: UnitId::MulazminII_5_1,
            cause: ElimCause::Combat,
        }
        .format_for_dispatch();
        assert!(line.starts_with("Mulazmin eliminated"), "{line}");
        let line = TurnEventRecord::Retreat {
            unit: UnitId::MulazminII_5_1,
            from: HexCoord::new(13, 5),
            to: HexCoord::new(14, 6),
        }
        .format_for_dispatch();
        assert_eq!(line, "Mulazmin retreated from (13, 5) to (14, 6)");
    }
}

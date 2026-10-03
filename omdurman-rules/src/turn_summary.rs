use serde::{Deserialize, Serialize};

use crate::DemolitionTarget;
use crate::effects::ElimCause;
use crate::turn_track::GameTime;
use crate::{
    CombatResult, DayNight, DieRoll, FireKind, FireModifier, GameTurnIndex, HexCoord, Player,
    UnitId, VictoryPoints, VpSource,
};
use omdurman_types::HexsideRef;

/// A single structured event recorded during a game turn.
///
/// Accumulated by `apply_effect` arms into `GameState::turn_events` and
/// snapshotted into a [`TurnSummary`] when the game turn advances.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TurnEventRecord {
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
    /// A howitzer shell impacted at `at` (§6.64) — `scattered` when the
    /// impact roll moved the shell off the aimed hex.
    HowitzerImpact { at: HexCoord, scattered: bool },
    /// Victory points were scored.
    VpScored {
        source: VpSource,
        points: VictoryPoints,
        for_player: Player,
    },
    /// Artillery fired to breach a wall hexside (§6.63) -- a CRT roll, but
    /// no fire combat: its result is the breach, not casualties.
    WallBreach {
        attacker: Player,
        firers: Vec<UnitId>,
        hexside: HexsideRef,
        roll: DieRoll,
        breached: bool,
        /// The adjacent enemy unit caught in a successful breach.
        eliminated: Option<UnitId>,
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
                format!("{for_player} scores {} VP: {source}", points.value())
            }
            TurnEventRecord::WallBreach {
                attacker,
                hexside,
                breached,
                eliminated,
                ..
            } => {
                let at = format!("{}-{}", hex(&hexside.a), hex(&hexside.b));
                match (breached, eliminated) {
                    (false, _) => format!("{attacker} artillery failed to breach the wall at {at}"),
                    (true, None) => format!("{attacker} artillery breached the wall at {at}"),
                    (true, Some(victim)) => format!(
                        "{attacker} artillery breached the wall at {at}; {} caught in the breach",
                        unit_name(victim)
                    ),
                }
            }
        }
    }
}

impl TurnSummary {
    /// Format the full turn as a structured text block for LLM input. In a
    /// scenario that keeps no victory points
    /// ([`Scenario::keeps_victory_points`](omdurman_types::Scenario::keeps_victory_points))
    /// the engine's VP bookkeeping never reaches the text.
    pub fn format_for_llm(&self, scenario: omdurman_types::Scenario) -> String {
        let mut out = format!(
            "=== Turn {} ({}, {:?}) ===\n",
            self.turn.0, self.time, self.day_night,
        );
        let no_vp = !scenario.keeps_victory_points();
        for event in &self.events {
            if no_vp && matches!(event, TurnEventRecord::VpScored { .. }) {
                continue;
            }
            out.push_str(&format!(
                "- {}\n",
                without_hexes(&event.format_for_dispatch())
            ));
        }
        out
    }
}

/// `line` without its hex coordinates ("(13, 5)") and the preposition
/// before each (" at", " from", " to", " on", a hexside's "-"): the model
/// quoted them back, and a reader cannot place them.
fn without_hexes(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(open) = rest.find('(') {
        let (before, from_paren) = rest.split_at(open);
        let close = from_paren.find(')');
        let is_hex = close.is_some_and(|close| {
            let inner = &from_paren[1..close];
            inner.split_once(", ").is_some_and(|(q, r)| {
                [q, r].iter().all(|n| {
                    !n.is_empty()
                        && n.trim_start_matches('-')
                            .chars()
                            .all(|c| c.is_ascii_digit())
                })
            })
        });
        match close {
            Some(close) if is_hex => {
                let mut kept = before;
                for prep in [" at ", " from ", " to ", " on ", "-"] {
                    if let Some(stripped) = kept.strip_suffix(prep) {
                        kept = stripped;
                        break;
                    }
                }
                out.push_str(kept);
                rest = &from_paren[close + 1..];
            }
            _ => {
                out.push_str(before);
                out.push('(');
                rest = &from_paren[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The model's copy of the turn carries no hex coordinates (it quoted
    /// them back); other parentheses survive.
    #[test]
    fn llm_lines_drop_hex_coordinates() {
        assert_eq!(
            without_hexes("Mulazmin retreated from (13, 5) to (14, 6)"),
            "Mulazmin retreated"
        );
        assert_eq!(
            without_hexes("Dervish fire at (40, 12): rolled 4 -> NoEffect"),
            "Dervish fire: rolled 4 -> NoEffect"
        );
        assert_eq!(
            without_hexes("Dervish artillery failed to breach the wall at (28, 38)-(28, 39)"),
            "Dervish artillery failed to breach the wall"
        );
        assert_eq!(
            without_hexes("Anglo-Egyptian reinforcements (Gunboat Naser) placed at (28, 0)"),
            "Anglo-Egyptian reinforcements (Gunboat Naser) placed"
        );
    }

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

    /// Only the Campaign keeps victory points: the Historical scenario
    /// (§9.24, units eliminated) and FALL OF KHARTOUM never feed VP lines to
    /// the telegraph, and tell it not to mention points.
    #[traceability_macro::rulebook("§9.24")]
    #[test]
    fn only_the_campaign_reports_victory_points() {
        use omdurman_types::Scenario;
        let summary = TurnSummary {
            turn: GameTurnIndex::new(1),
            time: crate::turn_track::GameTime::SixAM,
            day_night: DayNight::Day,
            first_player: Player::Dervish,
            events: vec![TurnEventRecord::VpScored {
                source: VpSource::DervishUnitEliminated,
                points: VpSource::DervishUnitEliminated.points(),
                for_player: Player::AngloEgyptian,
            }],
        };
        assert!(summary.format_for_llm(Scenario::Campaign).contains("VP"));
        for scenario in [Scenario::Historical, Scenario::FallOfKhartoum] {
            assert!(
                !summary.format_for_llm(scenario).contains("VP"),
                "{scenario:?}"
            );
            let (system, _) = crate::telegram_prompt::build_telegram_prompt(&summary, scenario);
            assert!(
                system.contains("never mention victory points"),
                "{scenario:?}"
            );
        }
    }
}

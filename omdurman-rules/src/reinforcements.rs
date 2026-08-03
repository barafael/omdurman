use omdurman_types::{DervishTribe, Player, Scenario, UnitKind};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{BritishLeader, DervishLeader};

/// Reinforcement entry points for the Campaign scenario (§9.112/§9.113).
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, EnumIter)]
pub enum ReinforcementEntry {
    /// Dervish: west edge, south of Khor Shambat (§9.112).
    DervishWestEdge,
    /// AE: Anglo-Egyptian Entrance Area on the west bank (§9.113).
    AngloEgyptianEntrance,
    /// AE: gunboats enter any north-edge Nile River hex (§9.113).
    GunboatNorthEdge,
    /// AE: "Friendlies" enter via the Abu Alim hut on the east bank (§9.113).
    AbuAlimHut,
}

/// The cost to cross from the off-board edge into the first hex.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct EntryCost(pub i16);

/// A single reinforcement wave: the units eligible to enter on a given turn.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReinforcementWave {
    /// 1-based turn index (Campaign: 1..=4 per §9.112/§9.113).
    pub turn: u8,
    /// Entry point for this wave.
    pub entry: ReinforcementEntry,
    /// Movement-point cost to enter the first hex.
    pub entry_cost: EntryCost,
    /// Maximum number of units (excluding leaders) that may enter this turn.
    /// `None` = no cap.
    pub unit_cap: Option<usize>,
    /// Leaders that may enter this turn (free, no cap).
    pub leaders: Vec<CampaignLeader>,
    /// Tribes whose sections may enter this turn.
    pub tribes: Vec<DervishTribe>,
    /// Whether all remaining units of the eligible types must enter this turn.
    pub all_remaining: bool,
}

/// Leaders that arrive as reinforcements in the Campaign scenario.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CampaignLeader {
    Dervish(DervishLeader),
    British(BritishLeader),
}

/// The full reinforcement schedule for one side in the Campaign scenario.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReinforcementSchedule {
    pub player: Player,
    pub waves: Vec<ReinforcementWave>,
}

impl ReinforcementSchedule {
    /// Get the wave for a given 1-based turn index, if one exists.
    pub fn wave_for_turn(&self, turn: u8) -> Option<&ReinforcementWave> {
        self.waves.iter().find(|w| w.turn == turn)
    }

    /// Whether this side has any reinforcements at all.
    pub fn has_reinforcements(&self) -> bool {
        !self.waves.is_empty()
    }
}

/// §9.112: Dervish Campaign reinforcement schedule.
///
/// All reinforcements enter on the west edge, south of the Khor Shambat.
/// Each unit pays terrain cost of the hex it enters through.
pub fn dervish_campaign_schedule() -> ReinforcementSchedule {
    ReinforcementSchedule {
        player: Player::Dervish,
        waves: vec![
            // Turn 1: Baggara, Jaalin, Danagla, Kehena, Degheim + Yakub, Sherif, Ali Wad Helu
            ReinforcementWave {
                turn: 1,
                entry: ReinforcementEntry::DervishWestEdge,
                entry_cost: EntryCost(1),
                unit_cap: None,
                leaders: vec![
                    CampaignLeader::Dervish(DervishLeader::Yakub),
                    CampaignLeader::Dervish(DervishLeader::Sherif),
                    CampaignLeader::Dervish(DervishLeader::AliWadHelu),
                ],
                tribes: vec![
                    DervishTribe::Baggara,
                    DervishTribe::Jaalin,
                    DervishTribe::Danagla,
                    DervishTribe::Kehena,
                    DervishTribe::Degheim,
                ],
                all_remaining: false,
            },
            // Turn 2: Hadendowa + Osman Digna
            ReinforcementWave {
                turn: 2,
                entry: ReinforcementEntry::DervishWestEdge,
                entry_cost: EntryCost(1),
                unit_cap: None,
                leaders: vec![CampaignLeader::Dervish(DervishLeader::OsmanDigna)],
                tribes: vec![DervishTribe::Hadendowa],
                all_remaining: false,
            },
            // Turn 3: Mulazmin, Jehadia + Sheik El Din
            ReinforcementWave {
                turn: 3,
                entry: ReinforcementEntry::DervishWestEdge,
                entry_cost: EntryCost(1),
                unit_cap: None,
                leaders: vec![CampaignLeader::Dervish(DervishLeader::SheikElDin)],
                tribes: vec![DervishTribe::Mulazmin, DervishTribe::Jehadia],
                all_remaining: true,
            },
        ],
    }
}

/// §9.113: Anglo-Egyptian Campaign reinforcement schedule.
///
/// Leaders (Kitchener, Gatacre, Hunter) may enter anytime during the first
/// four turns and do not count against the 12-unit-per-turn limit.  All three
/// leaders must be in play by the end of turn four.
///
/// - Gunboats enter north-edge Nile River hexes (1 MP first hex).
/// - "Friendlies" enter via Abu Alim hut on the east bank (8 MP per unit).
/// - All other AE units enter via the Anglo-Egyptian Entrance Area (1 MP).
pub fn anglo_egyptian_campaign_schedule() -> ReinforcementSchedule {
    let free_leaders = vec![
        CampaignLeader::British(BritishLeader::Kitchener),
        CampaignLeader::British(BritishLeader::Gatacre),
        CampaignLeader::British(BritishLeader::Hunter),
    ];

    ReinforcementSchedule {
        player: Player::AngloEgyptian,
        waves: vec![
            // Turn 1: up to 3 gunboats, Friendlies, Egyptian Cavalry,
            // Horse Artillery, 2 infantry brigades from Egyptian Division.
            ReinforcementWave {
                turn: 1,
                entry: ReinforcementEntry::AngloEgyptianEntrance,
                entry_cost: EntryCost(1),
                unit_cap: Some(12),
                leaders: free_leaders.clone(),
                tribes: vec![],
                all_remaining: false,
            },
            // Turn 2: up to 3 gunboats + any 12 land units.
            ReinforcementWave {
                turn: 2,
                entry: ReinforcementEntry::AngloEgyptianEntrance,
                entry_cost: EntryCost(1),
                unit_cap: Some(12),
                leaders: free_leaders.clone(),
                tribes: vec![],
                all_remaining: false,
            },
            // Turn 3: up to 3 gunboats + any 12 land units.
            ReinforcementWave {
                turn: 3,
                entry: ReinforcementEntry::AngloEgyptianEntrance,
                entry_cost: EntryCost(1),
                unit_cap: Some(12),
                leaders: free_leaders.clone(),
                tribes: vec![],
                all_remaining: false,
            },
            // Turn 4: all remaining Anglo-Egyptian units.
            ReinforcementWave {
                turn: 4,
                entry: ReinforcementEntry::AngloEgyptianEntrance,
                entry_cost: EntryCost(1),
                unit_cap: None,
                leaders: free_leaders.clone(),
                tribes: vec![],
                all_remaining: true,
            },
        ],
    }
}

/// Returns the reinforcement schedule for a given scenario, if applicable.
pub fn schedule_for_scenario(scenario: Scenario) -> Option<ReinforcementSchedule> {
    match scenario {
        Scenario::Campaign => {
            // Both sides have reinforcements in the Campaign scenario.
            // The caller picks which side they want via the returned schedule.
            None // Campaign has two schedules; use the specific functions.
        }
        // Historical and Fall of Khartoum have no reinforcement schedules --
        // all units are pre-placed.
        Scenario::Historical | Scenario::FallOfKhartoum => None,
    }
}

/// Whether a unit identity belongs to a Dervish tribe eligible for a given wave.
pub fn dervish_tribe_eligible(
    tribe: DervishTribe,
    wave: &ReinforcementWave,
) -> bool {
    wave.tribes.contains(&tribe)
}

/// Whether a unit kind is eligible for the Anglo-Egyptian reinforcement waves.
pub fn anglo_egyptian_unit_eligible(kind: UnitKind) -> bool {
    matches!(
        kind,
        UnitKind::Infantry { .. }
            | UnitKind::Cavalry { .. }
            | UnitKind::Artillery { .. }
            | UnitKind::Gunboat { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use traceability_macro::rulebook;

    #[rulebook("§9.112")]
    #[test]
    fn dervish_schedule_has_three_waves() {
        let sched = dervish_campaign_schedule();
        assert_eq!(sched.waves.len(), 3);
        assert_eq!(sched.waves[0].turn, 1);
        assert_eq!(sched.waves[1].turn, 2);
        assert_eq!(sched.waves[2].turn, 3);
    }

    #[rulebook("§9.112")]
    #[test]
    fn dervish_wave_one_has_baggaara_and_three_leaders() {
        let sched = dervish_campaign_schedule();
        let w = &sched.waves[0];
        assert!(w.tribes.contains(&DervishTribe::Baggara));
        assert_eq!(w.leaders.len(), 3);
    }

    #[rulebook("§9.112")]
    #[test]
    fn dervish_wave_two_has_hadendowa() {
        let sched = dervish_campaign_schedule();
        let w = &sched.waves[1];
        assert!(w.tribes.contains(&DervishTribe::Hadendowa));
        assert_eq!(w.leaders.len(), 1);
    }

    #[rulebook("§9.112")]
    #[test]
    fn dervish_wave_three_is_all_remaining() {
        let sched = dervish_campaign_schedule();
        assert!(sched.waves[2].all_remaining);
    }

    #[rulebook("§9.113")]
    #[test]
    fn anglo_egyptian_schedule_has_four_waves() {
        let sched = anglo_egyptian_campaign_schedule();
        assert_eq!(sched.waves.len(), 4);
        assert_eq!(sched.waves[0].turn, 1);
        assert_eq!(sched.waves[3].turn, 4);
    }

    #[rulebook("§9.113")]
    #[test]
    fn anglo_egyptian_leaders_available_each_wave() {
        let sched = anglo_egyptian_campaign_schedule();
        for w in &sched.waves {
            // Kitchener, Gatacre, Hunter available every turn (§9.113).
            let leaders: Vec<_> = w
                .leaders
                .iter()
                .filter(|l| matches!(l, CampaignLeader::British(_)))
                .collect();
            assert_eq!(leaders.len(), 3);
        }
    }

    #[rulebook("§9.113")]
    #[test]
    fn anglo_egyptian_turn_four_is_all_remaining() {
        let sched = anglo_egyptian_campaign_schedule();
        assert!(sched.waves[3].all_remaining);
    }

    #[rulebook("§9.112")]
    #[test]
    fn wave_for_turn_returns_correct_wave() {
        let sched = dervish_campaign_schedule();
        assert!(sched.wave_for_turn(1).is_some());
        assert!(sched.wave_for_turn(4).is_none());
    }

    #[test]
    fn historical_and_fok_have_no_schedule() {
        assert!(schedule_for_scenario(Scenario::Historical).is_none());
        assert!(schedule_for_scenario(Scenario::FallOfKhartoum).is_none());
    }
}

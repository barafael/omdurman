//! The human-readable game log that the offline observer reviews.
//!
//! One plain-text file: a header, one line per applied `GameEffect` with the
//! acting side, the engine's `Observation`s (carrying authoritative §
//! citations), turn-boundary summaries, and interleaved agent reasoning. The
//! log is the observer's ground truth — it must "give enough context" on its
//! own, with no live engine access.

use omdurman_rules::effects::{GameState, Observation};
use omdurman_rules::turn_summary::TurnSummary;
use omdurman_types::{Player, Scenario};

use crate::agent::Agents;
use crate::describe::{describe_observation, describe_turn_event};

/// The accumulated log. Rendered via [`GameLog::render`]; byte-stable for a
/// given seed (same seed → identical log), so the log is testable like the
/// event trace.
pub struct GameLog {
    lines: Vec<String>,
    events_logged: usize,
    observations_logged: usize,
    turn_boundaries: usize,
}

impl GameLog {
    /// Start a log with its header describing the run.
    pub fn new(scenario: Scenario, seed: u64, agents: &Agents) -> Self {
        let lines = vec![
            "GAME LOG — Remember Gordon! (The Battle of Omdurman)".to_string(),
            format!("scenario:        {}", scenario.label().to_lowercase()),
            format!("seed:            0x{seed:x}"),
            format!(
                "agents:          ae={} dervish={}",
                agents.label_for(Player::AngloEgyptian),
                agents.label_for(Player::Dervish),
            ),
            "rules_version:   Manual §1–§10".to_string(),
            String::new(),
        ];
        Self {
            lines,
            events_logged: 0,
            observations_logged: 0,
            turn_boundaries: 0,
        }
    }

    /// A line for one applied `GameEffect`.
    pub fn push_event(&mut self, seq: usize, turn: u8, phase: &str, actor: Player, text: &str) {
        self.lines
            .push(format!("[{seq}] T{turn} {phase} {actor}  {text}"));
        self.events_logged += 1;
    }

    /// An engine observation produced by the event at `seq`.
    pub fn push_observation(&mut self, seq: usize, obs: &Observation) {
        self.lines
            .push(format!("      → {}  [event {seq}]", describe_observation(obs)));
        self.observations_logged += 1;
    }

    /// Interleaved agent reasoning (LLM-advised sides only).
    pub fn push_reasoning(&mut self, side: Player, turn: u8, text: &str) {
        self.lines.push(format!("[reasoning, {side} T{turn}] {text}"));
    }

    /// A completed-game-turn boundary, from the engine's `TurnSummary`.
    /// `state` supplies the victory ledger for the running VP line.
    pub fn push_turn_boundary(&mut self, summary: &TurnSummary, state: &GameState) {
        self.turn_boundaries += 1;
        let (mut fire, mut melee, mut elims, mut advances, mut retreats, mut reinforcements) =
            (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
        for ev in &summary.events {
            use omdurman_rules::turn_summary::TurnEventRecord as R;
            match ev {
                R::FireCombat { .. } => fire += 1,
                R::MeleeCombat { .. } => melee += 1,
                R::UnitEliminated { .. } => elims += 1,
                R::AdvanceAfterCombat { .. } => advances += 1,
                R::Retreat { .. } => retreats += 1,
                R::Reinforcements { units, .. } => reinforcements += units.len(),
                _ => {}
            }
        }
        let ae_vp = state.victory.total_for(Player::AngloEgyptian).value();
        let d_vp = state.victory.total_for(Player::Dervish).value();
        self.lines.push(format!(
            "=== Turn {} complete ({}, {:?}) — {} fire, {} melee, {} eliminations, {} advances, {} retreats, {} reinforcements; VP AE {ae_vp} / Dervish {d_vp} ===",
            summary.turn.value(),
            summary.time,
            summary.day_night,
            fire,
            melee,
            elims,
            advances,
            retreats,
            reinforcements,
        ));
        for ev in &summary.events {
            self.lines.push(format!("    - {}", describe_turn_event(ev)));
        }
    }

    /// A driver annotation that is neither an event nor agent reasoning --
    /// e.g. a dropped LLM plan index or an illegal generated pick. These
    /// lines mark where an agent ran into the rules' boundaries.
    pub fn push_note(&mut self, turn: u8, text: &str) {
        self.lines.push(format!("[note, T{turn}] {text}"));
    }

    /// End-of-game footer with the typed result and final victory points.
    pub fn push_footer(&mut self, state: &GameState) {
        let ae_vp = state.victory.total_for(Player::AngloEgyptian).value();
        let d_vp = state.victory.total_for(Player::Dervish).value();
        let result = state
            .game_result
            .as_ref()
            .map(|r| format!("{r:?}"))
            .unwrap_or_else(|| "—".to_string());
        self.lines.push(String::new());
        self.lines.push(format!("=== GAME OVER ===  result: {result}"));
        self.lines.push(format!("victory: AE {ae_vp} / Dervish {d_vp}"));
    }

    /// Number of effect lines logged so far.
    pub fn events_logged(&self) -> usize {
        self.events_logged
    }

    /// Number of observation lines logged so far.
    pub fn observations_logged(&self) -> usize {
        self.observations_logged
    }

    /// Number of `=== Turn N complete ===` boundaries emitted so far.
    pub fn turn_boundaries(&self) -> usize {
        self.turn_boundaries
    }

    /// Render the full log as text (trailing newline included).
    pub fn render(&self) -> String {
        let mut out = self.lines.join("\n");
        out.push('\n');
        out
    }
}

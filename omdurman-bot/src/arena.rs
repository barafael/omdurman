//! Commander-vs-commander arena: plays whole games through the **app's**
//! AI decision path (`omdurman-app/src/bot_player.rs::next_ai_action`:
//! deep-setup / lean candidates, the mandatory §8.2 desertion roll first,
//! then engine-validated picks) with a chosen doctrine [`Version`] per side,
//! so the live commanders ([`crate::commanders`]) can be measured against
//! the frozen [`crate::baseline`] on the same seeds.
//!
//! Every game is reproducible from `(scenario, seed, versions)`: the only
//! randomness is the one `BotRng` the candidate enumerator rolls its dice
//! from (dice are embedded in the effects, as in the app).

use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
use omdurman_rules::{GameResult, HistoricalVictoryLevel, Phase};
use omdurman_types::{Player, Scenario};

use crate::rng::BotRng;

/// Which doctrine a side plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Version {
    /// The frozen pre-tuning commanders ([`crate::baseline`]).
    Baseline,
    /// The live commanders the app plays ([`crate::commanders`]).
    Current,
}

impl Version {
    pub fn name(self) -> &'static str {
        match self {
            Version::Baseline => "baseline",
            Version::Current => "current",
        }
    }

    fn pick(
        self,
        state: &GameState,
        player: Player,
        candidates: &[GameEffect],
        rng: &mut BotRng,
    ) -> GameEffect {
        match self {
            Version::Baseline => crate::baseline::pick_validated(state, player, candidates, rng),
            Version::Current => crate::commanders::pick_validated(state, player, candidates, rng),
        }
    }

    fn pick_setup(
        self,
        state: &GameState,
        candidates: &[GameEffect],
        own_side: Option<Player>,
        rng: &mut BotRng,
    ) -> GameEffect {
        match self {
            Version::Baseline => {
                crate::baseline::pick_setup_validated(state, candidates, own_side, rng)
            }
            Version::Current => {
                crate::commanders::pick_setup_validated(state, candidates, own_side, rng)
            }
        }
    }
}

/// The outcome of one arena game.
#[derive(Clone, Debug)]
pub struct ArenaResult {
    pub scenario: Scenario,
    pub seed: u64,
    pub ae: Version,
    pub dervish: Version,
    pub result: Option<GameResult>,
    pub game_over: bool,
    /// Last turn reached.
    pub turns: u8,
    pub vp_ae: i32,
    pub vp_dervish: i32,
    /// Anglo-Egyptian units eliminated (by the Dervish).
    pub ae_lost: i16,
    /// Dervish units eliminated (by the Anglo-Egyptians).
    pub dervish_lost: i16,
    /// Decisions taken (both sides).
    pub steps: usize,
    /// Slowest single decision, in milliseconds.
    pub max_decision_ms: f64,
    /// Mean decision time, in milliseconds.
    pub mean_decision_ms: f64,
    /// The Anglo-Egyptians held the Mahdi's Tomb at the end (§9.14).
    pub tomb_taken: bool,
}

impl ArenaResult {
    /// The result on one signed scale, positive = Anglo-Egyptian ahead:
    /// Campaign: the §9.14 VP superiority; Historical: the §9.24 net level
    /// (-4..4) times 10 plus the elimination balance as a tie-break; Fall of
    /// Khartoum: the §9.35 level (-3..3, no draw).
    pub fn ae_score(&self) -> i32 {
        match self.result {
            Some(GameResult::Campaign(_)) | None if self.scenario == Scenario::Campaign => {
                self.vp_ae - self.vp_dervish
            }
            Some(GameResult::Historical { ae, d }) => {
                let (winner, level) = HistoricalVictoryLevel::net(ae, d);
                let lv = level as i32 - 1;
                let signed = match winner {
                    Some(Player::AngloEgyptian) => lv,
                    Some(Player::Dervish) => -lv,
                    None => 0,
                };
                signed * 10 + (self.dervish_lost as i32 - 3 * self.ae_lost as i32).signum()
            }
            Some(GameResult::FoK(level)) => level as i32,
            _ => 0,
        }
    }

    /// Who won, if anyone.
    pub fn winner(&self) -> Option<Player> {
        match self.result? {
            GameResult::Campaign(level) => match level {
                omdurman_rules::CampaignVictoryLevel::Draw => None,
                omdurman_rules::CampaignVictoryLevel::Marginal(p)
                | omdurman_rules::CampaignVictoryLevel::Tactical(p)
                | omdurman_rules::CampaignVictoryLevel::Decisive(p) => Some(p),
            },
            GameResult::Historical { ae, d } => HistoricalVictoryLevel::net(ae, d).0,
            GameResult::FoK(level) => Some(if (level as i32) > 0 {
                Player::AngloEgyptian
            } else {
                Player::Dervish
            }),
        }
    }

    /// A one-line summary.
    pub fn line(&self) -> String {
        format!(
            "{:?} seed={} ae={} d={} turns={} over={} vp={}-{} lost ae={} d={} tomb={} score={:+} ms={:.1}/{:.0} result={}",
            self.scenario,
            self.seed,
            self.ae.name(),
            self.dervish.name(),
            self.turns,
            self.game_over,
            self.vp_ae,
            self.vp_dervish,
            self.ae_lost,
            self.dervish_lost,
            self.tomb_taken,
            self.ae_score(),
            self.mean_decision_ms,
            self.max_decision_ms,
            self.result
                .map(|r| r.display_key())
                .unwrap_or_else(|| "-".into()),
        )
    }
}

/// Hard ceiling on decisions per game (a stalled phase must never hang).
const MAX_STEPS: usize = 60_000;

/// Play one game, `ae` commanding the Anglo-Egyptians and `dervish` the
/// Dervish, through the app's decision path.
pub fn play(scenario: Scenario, seed: u64, ae: Version, dervish: Version) -> ArenaResult {
    let board = crate::playthrough::board_for_scenario(scenario);
    let mut state = GameState::with_board(scenario, board);
    let mut rng = BotRng::from_seed(seed);
    let version = |p: Player| match p {
        Player::AngloEgyptian => ae,
        Player::Dervish => dervish,
    };
    let mut steps = 0usize;
    let mut stuck = 0usize;
    let trace = std::env::var("ARENA_TRACE").is_ok();
    let started = std::time::Instant::now();
    let mut max_decision_ms = 0.0f64;
    let mut traced_turn = 0u8;
    while !state.game_over && steps < MAX_STEPS {
        steps += 1;
        if trace && state.current_turn.value() != traced_turn {
            traced_turn = state.current_turn.value();
            eprintln!("{}", trace_line(&state));
        }
        // The app's `ai_chooser`: the mine roll is the Dervish player's
        // (§10.12); otherwise the side to act (deployer in set-up, the
        // non-moving side in defensive fire).
        let decided = std::time::Instant::now();
        let chooser = if state.pending_mine.is_some() {
            Player::Dervish
        } else {
            state.player_to_act().unwrap_or(state.phase_player())
        };
        let candidates = if state.phase == Phase::Setup {
            crate::actions::legal_actions_deep_setup(&state, &mut rng)
        } else {
            crate::actions::legal_actions(&state, &mut rng)
        };
        let effect = if let Some(e) = candidates
            .iter()
            .find(|e| matches!(e, GameEffect::DervishDesertion { .. }))
        {
            match dervish {
                Version::Baseline => e.clone(),
                Version::Current => crate::commanders::choose_deserters(&state, e.clone()),
            }
        } else if state.phase == Phase::Setup {
            version(chooser).pick_setup(&state, &candidates, Some(chooser), &mut rng)
        } else {
            version(chooser).pick(&state, chooser, &candidates, &mut rng)
        };
        max_decision_ms = max_decision_ms.max(decided.elapsed().as_secs_f64() * 1000.0);
        if apply_effect(&mut state, &effect).is_err() {
            // The app retries next frame; here, force the phase on after a
            // run of rejections so a bad pick cannot stall the game.
            stuck += 1;
            if stuck > 3 {
                let _ = apply_effect(&mut state, &GameEffect::AdvancePhase);
                stuck = 0;
            }
        } else {
            stuck = 0;
        }
    }
    ArenaResult {
        scenario,
        seed,
        ae,
        dervish,
        result: state.game_result,
        game_over: state.game_over,
        turns: state.current_turn.value(),
        vp_ae: state.victory.total_for(Player::AngloEgyptian).value(),
        vp_dervish: state.victory.total_for(Player::Dervish).value(),
        ae_lost: state.victory.units_eliminated_by(Player::Dervish),
        dervish_lost: state.victory.units_eliminated_by(Player::AngloEgyptian),
        steps,
        max_decision_ms,
        mean_decision_ms: started.elapsed().as_secs_f64() * 1000.0 / steps.max(1) as f64,
        tomb_taken: state
            .victory
            .events
            .iter()
            .any(|e| e.source == omdurman_rules::VpSource::MahdisTombTaken),
    }
}

/// One diagnostic line per turn (`ARENA_TRACE`): unit counts, losses, and
/// the Anglo-Egyptian leaders' path cost to the Mahdi's Tomb.
fn trace_line(state: &GameState) -> String {
    let count = |p: Player| {
        state
            .units
            .iter()
            .filter(|u| u.profile.identity.owner() == p)
            .count()
    };
    let tomb = state
        .board
        .hex_of_location(omdurman_types::Location::MahdisTomb);
    let leaders: Vec<String> = state
        .units
        .iter()
        .filter(|u| {
            matches!(
                u.profile.identity,
                omdurman_rules::UnitIdentity::AngloEgyptianLeader(_)
            )
        })
        .map(|u| {
            let cost = tomb.and_then(|t| crate::threat::path_cost(state, u.position, t));
            format!("({},{})~{:?}", u.position.q, u.position.r, cost)
        })
        .collect();
    format!(
        "  turn {:>2} {:?}: ae={} d={} lost ae={} d={} leaders={}",
        state.current_turn.value(),
        state.day_night,
        count(Player::AngloEgyptian),
        count(Player::Dervish),
        state.victory.units_eliminated_by(Player::Dervish),
        state.victory.units_eliminated_by(Player::AngloEgyptian),
        leaders.join(" ")
    )
}

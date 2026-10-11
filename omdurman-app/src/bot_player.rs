//! In-game AI commanders: the host plays any faction whose seats are all
//! AI seats (committed in `StartGame`, or handed over later) (Kitchener for the Anglo-Egyptian, Khalifa for
//! the Dervish) through the *same* sequenced-event path a human uses.
//!
//! Why host-side only: the host-relay protocol gives a single global event
//! order, and `apply_effect` validates+mutates authoritatively. If every
//! peer ran the AI, duplicate submissions would collide in the sequenced
//! stream; a *host migration* hands the AI to the next host naturally, so
//! the invariant "the current host plays the AI factions" is the simple,
//! robust one. Guests and late joiners just see ordinary sequenced effects
//! (and the AI seats in `StartGame` tell the UI who is a machine).
//!
//! Pacing: one validated action per [`ACT_COOLDOWN_SECS`], so a spectator
//! can follow a commander-vs-commander game live; the effect stream is the
//! same one the replay timeline plays back afterwards. Every candidate is
//! validated against a cloned engine state before submission
//! ([`commanders::pick_validated`]), because the action enumerator's
//! predicates can be weaker than `apply_effect` and a rejected submission
//! would be recorded-then-failed — poison for the event log.

use bevy::prelude::*;

use omdurman_bot::actions::engine_accepts;
use omdurman_bot::commanders::{self, Commander};
use omdurman_bot::rng::BotRng;
use omdurman_net::{GameEvent, NetState};
use omdurman_rules::Phase;
use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_types::Player;

use crate::{PendingEdits, net_plugin};

/// The host-side AI driver's private state: its own dice stream (candidate
/// enumeration pre-rolls effect dice — the shared `GameRng` stays with the
/// human submission paths) and a per-action cooldown.
///
/// Always inserted (`init_resource` in `main.rs`): `bot_player_act` demands it
/// in every `InGame`+`Game` frame, and resources that blink in and out
/// (`sync_driver_seed` used to insert/remove it via deferred `commands`)
/// raced `bot_player_act`, which panicked with "Resource does not exist".
/// A fresh game replaces it via `apply_start_game` instead.
#[derive(Resource)]
pub struct BotDriver {
    rng: BotRng,
    /// Where each unit has stood this movement phase: no step back onto it
    /// (see `omdurman_bot::move_memory`).
    memory: omdurman_bot::move_memory::MoveMemory,
    cooldown: f32,
    /// Submission uid of the last action, until its sequenced echo confirms
    /// it. The engine state the next decision is computed from only reflects
    /// that action once it is applied, so the driver waits: deciding again on
    /// the stale state would submit a duplicate (or now-illegal) action under
    /// a fresh uid, which the host would sequence -- and the log record --
    /// as a second event.
    in_flight: Option<u64>,
    /// Seconds the action in flight has waited for its echo.
    in_flight_secs: f32,
    /// What the AI is waiting on, and for how long (see [`Wait`]).
    wait: Option<Wait>,
    /// The last decision was illegal even as the `AdvancePhase` fallback:
    /// the AI stands down (warned once) and looks again every
    /// [`STUCK_RETRY_SECS`] instead of re-submitting a rejected effect.
    stuck: bool,
    /// Turns finished when the driver last looked (`turn_summaries.len()`),
    /// and the seconds since that count last grew: the end-of-turn hold
    /// (see [`turn_end_hold`]).
    turns_seen: usize,
    since_turn_end: f32,
}

/// A driver on a fresh dice stream. The stream rolls the dice the AI's
/// effects carry (candidate enumeration pre-rolls them), so it is seeded
/// from fresh entropy like the app's own `GameRng`: a constant seed made the
/// AI repeat the same rolls in every game and after every rebuild. Replay
/// never runs the driver -- the effects in the log carry their dice -- so
/// nothing depends on its seed.
impl Default for BotDriver {
    fn default() -> Self {
        Self::from_seed(omdurman_net::new_seed())
    }
}

impl BotDriver {
    /// A driver on the dice stream of `seed` (tests pin one; the game uses
    /// [`Default`], fresh entropy).
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: BotRng::from_seed(seed),
            memory: Default::default(),
            cooldown: 0.0,
            in_flight: None,
            in_flight_secs: 0.0,
            wait: None,
            stuck: false,
            turns_seen: 0,
            since_turn_end: f32::INFINITY,
        }
    }
}

/// Seconds between AI actions, counted once the board has finished showing
/// the last one ([`crate::fx::BoardBusy`]). Purely presentational pacing:
/// fast enough to finish a game in minutes, slow enough to watch the assault
/// develop.
const ACT_COOLDOWN_SECS: f32 = 0.4;
/// How long the AI asks for display-rate frames while its action's echo is
/// on its way (normally a frame or two). A longer wait is the network's, and
/// waits on the idle wake-ups (see [`crate::activity`]) instead of spinning.
const IN_FLIGHT_SPIN_SECS: f32 = 0.5;
/// How often a stuck AI (no legal action) looks again.
const STUCK_RETRY_SECS: f32 = 2.0;
/// How long the AI may wait on one gate before saying so (once): the gates
/// are silent by design -- the human's turn, a pause, a telegram -- and an
/// AI waiting on one of them for good looked exactly like a hung one.
const WAIT_REPORT_SECS: f32 = 10.0;
/// After a turn ends the AI holds this long before playing on, so the
/// players can take in the turn's telegram (each peer shows it over its own
/// board).
const TURN_END_PAUSE_SECS: f32 = 3.0;
/// The longest the AI waits, after a turn ends, for the host's own player to
/// dismiss the telegram (see [`turn_end_hold`]).
const TELEGRAM_WAIT_MAX_SECS: f32 = 60.0;

/// What holds the AI after a turn ended `since_turn_end` seconds ago, if
/// anything. Every peer's players get [`TURN_END_PAUSE_SECS`] to read the
/// telegram. The host's own unread telegram holds it longer -- but only when
/// `host_reading`: the host holds a seat and is looking at the board. The
/// host is only the lowest peer id: a spectator, a teammate in the menu or a
/// player reviewing the timeline would otherwise hold the game up for
/// everyone. Capped at [`TELEGRAM_WAIT_MAX_SECS`].
fn turn_end_hold(since_turn_end: f32, host_reading: bool) -> Option<&'static str> {
    if since_turn_end < TURN_END_PAUSE_SECS {
        Some("the players to read the end-of-turn telegram")
    } else if host_reading && since_turn_end < TELEGRAM_WAIT_MAX_SECS {
        Some("the end-of-turn telegram to be read")
    } else {
        None
    }
}

/// A gate the AI is waiting on: which one, for how long, and whether it
/// has been reported. Reset whenever the gate changes or an action goes out.
struct Wait {
    gate: &'static str,
    secs: f32,
    reported: bool,
}

impl BotDriver {
    /// The AI is held by `gate` this frame: count the wait, and log it once
    /// after [`WAIT_REPORT_SECS`] with the state it waits in.
    fn waiting(&mut self, gate: &'static str, dt: f32, state: &GameState) {
        let wait = match &mut self.wait {
            Some(wait) if wait.gate == gate => wait,
            _ => self.wait.insert(Wait {
                gate,
                secs: 0.0,
                reported: false,
            }),
        };
        wait.secs += dt;
        if !wait.reported && wait.secs >= WAIT_REPORT_SECS {
            wait.reported = true;
            info!(
                gate,
                secs = format_args!("{:.0}", wait.secs),
                phase = ?state.phase,
                active = ?state.player_to_act(),
                "AI: waiting"
            );
        }
    }
}

/// The host's own view: which screen is up and who the host is, for the
/// holds that are about what the host sees (its board's animations, its own
/// telegram).
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct HostView<'w> {
    screen: Option<Res<'w, State<crate::Screen>>>,
    key: Option<Res<'w, crate::seats::LocalPlayerKey>>,
    telegrams: Option<Res<'w, crate::telegram::TelegramLog>>,
}

impl HostView<'_> {
    /// The host's board is on screen (its animations pace the AI).
    fn on_board(&self) -> bool {
        self.screen
            .as_ref()
            .is_some_and(|s| *s.get() == crate::Screen::Board)
    }

    /// The host's own seated player looks at the board and has not yet
    /// dismissed the latest turn's telegram.
    fn reading(&self, seats: &crate::seats::Seats, state: &GameState) -> bool {
        let seated = self
            .key
            .as_ref()
            .is_some_and(|key| crate::seats::seat_of(&seats.0, key.0).is_some());
        seated
            && self.on_board()
            && self
                .telegrams
                .as_ref()
                .is_some_and(|log| log.awaiting_ack(state))
    }
}

/// The host plays the AI factions' turns: enumerate legal actions, take the
/// commander's best *engine-validated* one, submit it as an ordinary game
/// event. Runs in the game states (Setup included — the AI deploys its own
/// force), paced by [`BotDriver`]'s cooldown -- on whatever screen the host
/// is, against the live state (also under a review, `timeline::LiveGame`).
#[allow(clippy::too_many_arguments)]
pub fn bot_player_act(
    time: Res<Time>,
    net: Res<NetState>,
    mut driver: ResMut<BotDriver>,
    seats: crate::seats::SeatView,
    live: crate::timeline::LiveGame,
    mut pending: ResMut<PendingEdits>,
    offline: Option<Res<net_plugin::OfflineMode>>,
    host: HostView,
    mut activity: ResMut<crate::activity::Activity>,
    board: crate::fx::BoardBusy,
) {
    let ai = crate::seats::ai_factions(&seats.seats.0);
    let state = live.state();
    let dt = time.delta_secs();
    // The end-of-turn clock runs whatever the gates below say.
    let turns = state.turn_summaries.len();
    if turns != driver.turns_seen {
        driver.turns_seen = turns;
        driver.since_turn_end = 0.0;
    } else {
        driver.since_turn_end += dt;
    }
    // Every gate below is silent by design; `waiting` says once, after a
    // while, which one holds the AI (see `WAIT_REPORT_SECS`).
    if ai.is_empty() {
        return driver.waiting("no AI seat", dt, state);
    }
    // Paused waiting for an absent seat holder: the AI waits too.
    if seats.presence.paused() {
        return driver.waiting("the game paused for an absent player", dt, state);
    }
    // A turn just ended: give the players its telegram first.
    if turns > 0
        && !state.game_over
        && let Some(gate) = turn_end_hold(driver.since_turn_end, host.reading(&seats.seats, state))
    {
        return driver.waiting(gate, dt, state);
    }
    // Only the host (or an offline self-hosted instance) drives the AI.
    if !(net.is_host || offline.is_some_and(|o| o.0)) {
        return driver.waiting("the host (this peer is a guest)", dt, state);
    }
    // Frames at the display rate only for the short, bounded waits (the
    // echo of the last action, the cooldown): every other wait -- a stalled
    // network, a stuck AI, the human's turn -- runs on the idle wake-ups.
    // One action in flight at a time (see `BotDriver::in_flight`).
    if let Some(uid) = driver.in_flight {
        if pending.unconfirmed.iter().any(|(u, _)| *u == uid) {
            driver.in_flight_secs += dt;
            if driver.in_flight_secs < IN_FLIGHT_SPIN_SECS {
                activity.keep_running();
            }
            return driver.waiting("the echo of the last action", dt, state);
        }
        driver.in_flight = None;
    }
    if state.game_over {
        return;
    }
    if state.board.terrain.is_empty() {
        return driver.waiting("a board to be loaded", dt, state);
    }
    let Some(chooser) = ai_chooser(state).filter(|side| ai.contains(side)) else {
        if omdurman_bot::actions::retreat_window_open(state) {
            return driver.waiting("the defender's retreat-before-melee decision", dt, state);
        }
        return driver.waiting("a human player to act", dt, state);
    };

    // Let the board finish showing the last action (a glide, a shot, a
    // ghost) before the next: the cooldown only runs on a quiet board, so
    // the AI's turn reads one action at a time instead of overlapping. (The
    // animations keep the frames coming meanwhile.) Only the host's own
    // board: off it (the menu, a review) nothing of the live game animates.
    if host.on_board() && board.busy() {
        return driver.waiting("the board to finish animating", dt, state);
    }
    driver.wait = None;
    driver.cooldown -= dt;
    if driver.cooldown > 0.0 {
        if !driver.stuck {
            activity.keep_running();
        }
        return;
    }

    let driver = &mut *driver;
    let effect = next_ai_action(state, chooser, &ai, &mut driver.rng, &mut driver.memory);
    // The commanders fall back to `AdvancePhase` when nothing else is legal;
    // if even that is refused, submitting it would only be rejected on the
    // echo, and the AI would pick it again and again. Every other decision
    // was already dry-run against this very state (see `next_ai_action`), so
    // only the unvalidated fallback is checked again.
    if effect == GameEffect::AdvancePhase && !engine_accepts(state, &effect) {
        if !driver.stuck {
            warn!(?chooser, ?effect, "AI: no legal action; standing down");
            driver.stuck = true;
        }
        driver.cooldown = STUCK_RETRY_SECS;
        return;
    }
    if driver.stuck {
        info!(?chooser, "AI: a legal action again; resuming");
        driver.stuck = false;
    }
    driver.cooldown = ACT_COOLDOWN_SECS;
    driver.in_flight = Some(pending.submit_game(GameEvent::Effect(effect)));
    driver.in_flight_secs = 0.0;
    activity.keep_running();
}

/// The side whose decision the AI would make now: the Dervish player's roll
/// for a mine a British gunboat struck (§10.12); the defender of a declared
/// melee while its retreat is open -- it reacts before the attacker resolves
/// (§7.5), so an AI attacker waits for a human defender to retreat or stand;
/// during set-up the side deploying -- set-up is sequential and in the
/// Campaign the Dervish deploy first though the Anglo-Egyptians move first
/// (§9.111/§9.113); otherwise the phase player (defensive fire belongs to the
/// non-moving side, §6.7). See [`omdurman_bot::actions::decider`].
fn ai_chooser(state: &GameState) -> Option<Player> {
    omdurman_bot::actions::decider(state)
}

/// One AI decision for `chooser` in `state`, engine-validated -- all but the
/// `AdvancePhase` fallback, which is returned unchecked. Setup is
/// special: the candidates mix both sides' deployments, so when both sides
/// are AI they are scored per owning commander; when a human holds the other
/// faction, the AI only touches its own units (a human's deployment, ready
/// latch and re-arrangements are theirs).
pub(crate) fn next_ai_action(
    state: &GameState,
    chooser: Player,
    ai_factions: &[Player],
    rng: &mut BotRng,
    memory: &mut omdurman_bot::move_memory::MoveMemory,
) -> GameEffect {
    let both_ai = ai_factions.len() == 2;
    let own_only = if state.phase == Phase::Setup && !both_ai {
        Some(chooser)
    } else {
        None
    };

    let candidates = if state.phase == Phase::Setup {
        omdurman_bot::actions::legal_actions_deep_setup(state, rng)
    } else {
        omdurman_bot::actions::legal_actions(state, rng)
    };

    // §8.2: the once-per-game desertion roll is mandatory — take it before
    // any other action (the in-game counterpart of the playthrough rule).
    if let Some(effect) = candidates
        .iter()
        .find(|e| matches!(e, GameEffect::DervishDesertion { .. }))
    {
        // Who deserts is the Khalifa's choice (§8.2); the roll stands.
        return validated(state, commanders::choose_deserters(state, effect.clone()));
    }

    if state.phase == Phase::Setup {
        return commanders::pick_setup_validated(state, &candidates, own_only, rng);
    }
    commanders::pick_validated(state, chooser, &candidates, rng, memory)
}

/// Validate one candidate on a cloned state; fall back to `AdvancePhase`
/// (offered whenever the phase may lawfully end). Mirrors
/// [`commanders::pick_validated`]'s safety net for the mandatory-action
/// shortcuts above.
fn validated(state: &GameState, effect: GameEffect) -> GameEffect {
    if engine_accepts(state, &effect) {
        return effect;
    }
    GameEffect::AdvancePhase
}

/// Display name of the AI commander of `player`'s faction in `scenario`,
/// for the lobby roster ("AI · Kitchener"). In 1885 the sides were
/// Gordon's and the Mahdi's, not Kitchener's and the Khalifa's.
pub fn commander_name_for(player: Player, scenario: omdurman_types::Scenario) -> &'static str {
    match (scenario, player) {
        (omdurman_types::Scenario::FallOfKhartoum, Player::AngloEgyptian) => "Gordon",
        (omdurman_types::Scenario::FallOfKhartoum, Player::Dervish) => "the Mahdi",
        _ => Commander::for_player(player).name(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_bot::agent::{AgentStrategy, Agents};
    use omdurman_rules::effects::apply_effect;

    /// Set-up is sequential and its order is not the turn order: in the
    /// Campaign the Dervish deploy first (§9.111) though the Anglo-Egyptians
    /// move first (§9.113). An AI Dervish facing a human Anglo-Egyptian used
    /// to wait on the active (first-moving) player and never deployed --
    /// found in a click-through play-test.
    #[test]
    fn the_ai_deploys_when_its_side_is_setting_up() {
        let mut state = GameState::new(omdurman_types::Scenario::Campaign);
        state.phase = Phase::Setup;
        state.active_player = Player::AngloEgyptian;
        assert_eq!(ai_chooser(&state), Some(Player::Dervish));
        state.setup_ready_dervish = true;
        assert_eq!(ai_chooser(&state), Some(Player::AngloEgyptian));
    }

    /// The end-of-turn hold: a short pause for everyone, and the host's own
    /// unread telegram only while the host is a seated player on the board
    /// -- a spectator host, or one in the menu or a review, never holds the
    /// game up -- and never for long.
    #[test]
    fn the_turn_end_holds_the_ai_briefly() {
        assert!(turn_end_hold(0.0, false).is_some());
        assert!(turn_end_hold(TURN_END_PAUSE_SECS - 0.1, false).is_some());
        assert!(turn_end_hold(TURN_END_PAUSE_SECS, false).is_none());
        assert!(turn_end_hold(TURN_END_PAUSE_SECS + 5.0, true).is_some());
        assert!(turn_end_hold(TELEGRAM_WAIT_MAX_SECS, true).is_none());
        assert!(turn_end_hold(f32::INFINITY, true).is_none());
    }

    /// §7.5: during a declared melee the defender decides first while it
    /// may retreat; an AI attacker facing a human defender therefore waits.
    #[test]
    fn the_defender_chooses_first_in_a_melee() {
        let script = omdurman_rules::tactics::all_scripts()
            .into_iter()
            .find(|s| s.name == "retreat_before_melee")
            .expect("the §7.5 vignette");
        let mut state = script.state;
        let omdurman_rules::tactics::ScriptStep::Legal { effect, .. } = &script.steps[0] else {
            panic!("the vignette opens with the declaration");
        };
        apply_effect(&mut state, effect).unwrap();
        // An Anglo-Egyptian attack on a Dervish camel unit.
        assert_eq!(ai_chooser(&state), Some(Player::Dervish));
        // A Kitchener attacker never plays the Dervish retreat for them.
        let mut rng = BotRng::from_seed(11);
        let mut memory = omdurman_bot::move_memory::MoveMemory::new();
        let pick = next_ai_action(
            &state,
            Player::AngloEgyptian,
            &[Player::AngloEgyptian],
            &mut rng,
            &mut memory,
        );
        assert!(!matches!(
            pick,
            GameEffect::RetreatBeforeMelee { .. } | GameEffect::DeclineRetreatBeforeMelee
        ));
        apply_effect(&mut state, &GameEffect::DeclineRetreatBeforeMelee).unwrap();
        assert_eq!(ai_chooser(&state), Some(Player::AngloEgyptian));
    }

    /// Every game's AI rolls fresh dice (review 2026-09, item 2.5): the driver
    /// `apply_start_game` installs used to restart one constant stream, so
    /// every game -- and every rebuild -- repeated the AI's rolls.
    #[test]
    fn every_game_gets_fresh_ai_dice() {
        let rolls =
            |driver: &mut BotDriver| -> Vec<_> { (0..24).map(|_| driver.rng.roll_d10()).collect() };
        assert_ne!(
            rolls(&mut BotDriver::default()),
            rolls(&mut BotDriver::default())
        );
        // An explicit seed still pins a stream (tests, fixtures).
        assert_eq!(
            rolls(&mut BotDriver::from_seed(7)),
            rolls(&mut BotDriver::from_seed(7))
        );
    }

    /// The [`Agents`] pair describing an AI configuration — what the headless
    /// tuning preset (`play <scenario> <seed> commanders`) runs, so tuning
    /// sessions and in-app games use one code path.
    fn agents_for(ai: &[Player]) -> Agents {
        let to_strategy = |p: Player| {
            if ai.contains(&p) {
                AgentStrategy::Commander(Commander::for_player(p))
            } else {
                AgentStrategy::Random
            }
        };
        Agents {
            ae: to_strategy(Player::AngloEgyptian),
            dervish: to_strategy(Player::Dervish),
        }
    }

    /// The in-app AI configuration maps onto the headless tuning preset
    /// (`play <scenario> <seed> commanders`): both factions AI = both
    /// commanders, one faction AI = that commander vs a random opponent.
    #[test]
    fn agents_for_mirrors_the_commanders_preset() {
        let both = agents_for(&[Player::AngloEgyptian, Player::Dervish]);
        assert!(matches!(
            both.ae,
            AgentStrategy::Commander(Commander::Kitchener)
        ));
        assert!(matches!(
            both.dervish,
            AgentStrategy::Commander(Commander::Khalifa)
        ));
        let one = agents_for(&[Player::Dervish]);
        assert!(matches!(one.ae, AgentStrategy::Random));
        assert!(matches!(
            one.dervish,
            AgentStrategy::Commander(Commander::Khalifa)
        ));
        let none = agents_for(&[]);
        assert!(matches!(none.ae, AgentStrategy::Random));
        assert!(matches!(none.dervish, AgentStrategy::Random));
    }

    /// Commander naming is stable — the lobby roster renders it.
    #[test]
    fn commander_names() {
        use omdurman_types::Scenario;
        assert_eq!(
            commander_name_for(Player::AngloEgyptian, Scenario::Campaign),
            "Kitchener"
        );
        assert_eq!(
            commander_name_for(Player::Dervish, Scenario::Historical),
            "Khalifa"
        );
        assert_eq!(
            commander_name_for(Player::AngloEgyptian, Scenario::FallOfKhartoum),
            "Gordon"
        );
    }

    /// The exact decision path [`bot_player_act`] uses —
    /// `pick_setup_validated` in Setup, `pick_validated` afterwards, both
    /// against deep-setup/lean candidate lists — must play a complete
    /// commander-vs-commander game where every submitted effect applies and
    /// the scenario reaches a resolution. This is the in-app Kitchener vs
    /// Khalifa game, minus rendering and pacing.
    #[test]
    fn app_decision_path_plays_out_kitchener_vs_khalifa() {
        use omdurman_rules::board::BoardInfo;
        use omdurman_rules::effects::GameState;
        use omdurman_types::Scenario;

        for seed in [7u64, 2026] {
            let board: BoardInfo = {
                let map = omdurman_rules::board_data::fall_of_khartoum_map_data();
                BoardInfo::from_map_data(map)
            };
            let mut state = GameState::with_board(Scenario::FallOfKhartoum, board);
            let mut rng = BotRng::from_seed(seed);
            let ai = [Player::AngloEgyptian, Player::Dervish];

            let mut steps = 0usize;
            let mut memory = omdurman_bot::move_memory::MoveMemory::new();
            while !state.game_over && state.current_turn.value() <= 12 && steps < 20_000 {
                steps += 1;
                let chooser = state.phase_player();
                assert!(
                    ai.contains(&chooser),
                    "in an AI-vs-AI game every chooser is an AI faction"
                );
                let candidates = if state.phase == Phase::Setup {
                    omdurman_bot::actions::legal_actions_deep_setup(&state, &mut rng)
                } else {
                    omdurman_bot::actions::legal_actions(&state, &mut rng)
                };
                let effect = if let Some(e) = candidates
                    .iter()
                    .find(|e| matches!(e, GameEffect::DervishDesertion { .. }))
                {
                    validated(&state, e.clone())
                } else if state.phase == Phase::Setup {
                    commanders::pick_setup_validated(&state, &candidates, None, &mut rng)
                } else {
                    commanders::pick_validated(&state, chooser, &candidates, &mut rng, &mut memory)
                };
                apply_effect(&mut state, &effect).unwrap_or_else(|e| {
                    panic!("seed {seed}: the validated pick {effect:?} was rejected: {e}")
                });
            }
            assert!(
                state.game_over,
                "seed {seed}: the in-app AI game did not reach a resolution"
            );
            assert!(
                state.game_result.is_some(),
                "seed {seed}: game over without a FoK result"
            );
        }
    }
}

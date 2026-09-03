//! In-game AI commanders: the host plays any faction committed to an AI
//! commander in `StartGame` (Kitchener for the Anglo-Egyptian, Khalifa for
//! the Dervish) through the *same* sequenced-event path a human uses.
//!
//! Why host-side only: the host-relay protocol gives a single global event
//! order, and `apply_effect` validates+mutates authoritatively. If every
//! peer ran the AI, duplicate submissions would collide in the sequenced
//! stream; a *host migration* hands the AI to the next host naturally, so
//! the invariant "the current host plays the AI factions" is the simple,
//! robust one. Guests and late joiners just see ordinary sequenced effects
//! (and the `StartGame.ai` list tells the UI who is a machine).
//!
//! Pacing: one validated action per [`ACT_COOLDOWN_SECS`], so a spectator
//! can follow a commander-vs-commander game live; the effect stream is the
//! same one the replay timeline plays back afterwards. Every candidate is
//! validated against a cloned engine state before submission
//! ([`commanders::pick_validated`]), because the action enumerator's
//! predicates can be weaker than `apply_effect` and a rejected submission
//! would be recorded-then-failed — poison for the event log.

use bevy::prelude::*;

use omdurman_bot::commanders::{self, Commander};
use omdurman_bot::rng::BotRng;
use omdurman_net::{GameEvent, NetState};
use omdurman_rules::Phase;
use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
use omdurman_types::Player;

use crate::{GameStateResource, PendingEdits, net_plugin};

/// The factions currently commanded by the AI (committed via `StartGame`).
#[derive(Resource, Default)]
pub struct AiCommanders(pub Vec<Player>);

/// The host-side AI driver's private state: its own dice stream (candidate
/// enumeration pre-rolls effect dice — the shared `GameRng` stays with the
/// human submission paths) and a per-action cooldown.
#[derive(Resource)]
pub struct BotDriver {
    rng: BotRng,
    cooldown: f32,
}

impl BotDriver {
    /// Seeded from a fixed constant: the submitted *effects* carry their own
    /// dice, so the driver's stream only shapes which legal action is picked,
    /// and a fresh game must not inherit a skewed stream.
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: BotRng::from_seed(seed),
            cooldown: 0.0,
        }
    }
}

/// Seconds between AI actions. Purely presentational pacing: fast enough to
/// finish a game in minutes, slow enough to watch the assault develop.
const ACT_COOLDOWN_SECS: f32 = 0.4;

/// The host plays the AI factions' turns: enumerate legal actions, take the
/// commander's best *engine-validated* one, submit it as an ordinary game
/// event. Runs in the game states (Setup included — the AI deploys its own
/// force), paced by [`BotDriver`]'s cooldown.
pub fn bot_player_act(
    time: Res<Time>,
    net: Res<NetState>,
    mut driver: ResMut<BotDriver>,
    ai: Res<AiCommanders>,
    game_state: Option<Res<GameStateResource>>,
    mut pending: ResMut<PendingEdits>,
) {
    if ai.0.is_empty() || game_state.is_none() {
        return;
    }
    // Only the host (or an offline self-hosted instance) drives the AI.
    if !(net.is_host || net_plugin::offline_mode()) {
        return;
    }
    let state = &game_state.unwrap().0;
    if state.game_over || state.board.terrain.is_empty() {
        return;
    }

    driver.cooldown -= time.delta_secs();
    if driver.cooldown > 0.0 {
        return;
    }

    // The acting side: the active player, except defensive fire where the
    // non-moving player fires (§6.7).
    let chooser = match state.phase {
        Phase::DefensiveFire(_) => state.active_player.opponent(),
        _ => state.active_player,
    };
    if !ai.0.contains(&chooser) {
        return;
    }

    let effect = next_ai_action(state, chooser, &ai.0, &mut driver.rng);
    driver.cooldown = ACT_COOLDOWN_SECS;
    pending.submit_game(GameEvent::Effect(effect));
}

/// One AI decision for `chooser` in `state`, engine-validated. Setup is
/// special: the candidates mix both sides' deployments, so when both sides
/// are AI they are scored per owning commander; when a human holds the other
/// faction, the AI only touches its own units (a human's deployment, ready
/// latch and re-arrangements are theirs).
fn next_ai_action(
    state: &GameState,
    chooser: Player,
    ai_factions: &[Player],
    rng: &mut BotRng,
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
        return validated(state, effect.clone());
    }

    if state.phase == Phase::Setup {
        return commanders::pick_setup_validated(state, &candidates, own_only, rng);
    }
    commanders::pick_validated(state, chooser, &candidates, rng)
}

/// Validate one candidate on a cloned state; fall back to `AdvancePhase`
/// (offered whenever the phase may lawfully end). Mirrors
/// [`commanders::pick_validated`]'s safety net for the mandatory-action
/// shortcuts above.
fn validated(state: &GameState, effect: GameEffect) -> GameEffect {
    let mut test = state.clone();
    if apply_effect(&mut test, &effect).is_ok() {
        return effect;
    }
    GameEffect::AdvancePhase
}

/// Seed/reset the AI driver when a game starts or the AI list empties.
pub fn sync_driver_seed(
    mut commands: Commands,
    ai: Res<AiCommanders>,
    existing: Option<Res<BotDriver>>,
) {
    if ai.is_changed() {
        if ai.0.is_empty() {
            if existing.is_some() {
                commands.remove_resource::<BotDriver>();
            }
        } else if existing.is_none() {
            // O-M-D-U-R-M-A-N — cosmetic only (see `BotDriver::from_seed`).
            commands.insert_resource(BotDriver::from_seed(0x4F4D_4455_524D_414E));
        }
    }
}

/// Display name of the AI commander of `player`'s faction, for the lobby
/// roster and faction banners ("AI · Kitchener").
pub fn commander_name_for(player: Player) -> &'static str {
    Commander::for_player(player).name()
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_bot::agent::{AgentStrategy, Agents};

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
        assert_eq!(commander_name_for(Player::AngloEgyptian), "Kitchener");
        assert_eq!(commander_name_for(Player::Dervish), "Khalifa");
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
                BoardInfo::from_map_data(&map)
            };
            let mut state = GameState::with_board(Scenario::FallOfKhartoum, board);
            let mut rng = BotRng::from_seed(seed);
            let ai = [Player::AngloEgyptian, Player::Dervish];

            let mut steps = 0usize;
            while !state.game_over && state.current_turn.value() <= 12 && steps < 20_000 {
                steps += 1;
                let chooser = match state.phase {
                    Phase::DefensiveFire(_) => state.active_player.opponent(),
                    _ => state.active_player,
                };
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
                    commanders::pick_validated(&state, chooser, &candidates, &mut rng)
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

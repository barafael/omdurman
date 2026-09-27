//! The single `GameEvent` application path, shared by the live receive path
//! (`net_socket::handle_socket`, on the host-sequenced echo) and history
//! replay / timeline scrub (`timeline::rebuild_state_to`).
//!
//! Every recorded variant -- `StartGame`, `Effect`, the sprite-shaped
//! `PlaceUnit` / `MoveUnit` / `RemoveUnit`, and the seat events
//! `SeatAssigned` / `SeatCarved` (which only touch the seat table) -- is applied
//! *synchronously, in sequence order*, through [`apply_game_event`]. The engine
//! state is therefore a pure function of the event log: a live peer and a
//! replaying late joiner apply exactly the same effects in exactly the same
//! order. The board sprites are a projection of that engine state
//! (`picker::reconcile_unit_sprites`), never a second application path.

use bevy::prelude::*;
use omdurman_net::GameEvent;
use omdurman_rules::OptionalRule;
use omdurman_rules::board::BoardInfo;
use omdurman_rules::effects::{GameEffect, GameState, apply_effect};
use omdurman_rules::{Phase, UnitId, UnitPlacement, UnitState};
use omdurman_types::{HexCoord, MapKind, Scenario, SpriteRef};

use crate::picker::UnitPaths;

/// Everything applying a `GameEvent` may mutate, as plain borrows so the live
/// socket path, the rebuild path and the tests share one function.
pub(crate) struct EventSinks<'a> {
    pub game_state: &'a mut GameState,
    /// The committed seat table (written only here, so live and replay
    /// agree on who holds which seat).
    pub seats: &'a mut crate::seats::Seats,
    pub local_setup_ready: &'a mut crate::peers::LocalSetupReady,
    pub bot_driver: &'a mut crate::bot_player::BotDriver,
    pub loaded_annotations: &'a mut crate::board_state::LoadedAnnotations,
    pub pending_map_load: &'a mut crate::board_state::PendingMapLoad,
    /// Per-turn movement routes (drawn as arrows), recorded at the one point
    /// where a move is accepted, so live, remote and replayed moves of both
    /// factions all show up.
    pub unit_paths: &'a mut UnitPaths,
    /// The telegrams and Gazette filed from recorded press events.
    pub press: &'a mut crate::telegram::TelegramLog,
}

/// The host-committed fields of a `GameEvent::StartGame`, passed through
/// to [`apply_start_game`] as a bundle.
pub(crate) struct StartGameFields<'a> {
    /// The seat table (humans by stable key, plus AI seats).
    pub seats: &'a [omdurman_net::Seat],
    pub scenario: Scenario,
    pub optional_rules: &'a [OptionalRule],
}

/// State core of a `StartGame`:
///
/// * install the committed seat table (§1.1 command scopes ride in it);
/// * seed a fresh engine state — `GameState::new` sets the scenario's
///   first-moving player (§9.113/§9.212/§9.322) — and push the committed
///   optional rules (§10.11/§10.21);
/// * attach the scenario's board to the engine state *synchronously*, so
///   movement costing / ZOC never validate against an empty board between
///   `StartGame` and the deferred visual map load;
/// * stage the *visual* board load for the next frame (§dual-map).
///
/// Caller-specific concerns (mode switches, snapshot requests) stay with the
/// callers.
pub(crate) fn apply_start_game(fields: StartGameFields<'_>, sinks: &mut EventSinks<'_>) -> MapKind {
    let StartGameFields {
        seats,
        scenario,
        optional_rules,
    } = fields;
    // Seats ride in StartGame, so replays and late joiners gate on the same
    // seats (and §1.1 command scopes) the host started with.
    sinks.seats.0 = seats.to_vec();
    // A fresh game restarts per-member setup readiness (§9.2/§9.3).
    sinks.local_setup_ready.0 = false;
    // A fresh game must not inherit a skewed driver stream: the submitted
    // *effects* carry their own dice, but the driver's private stream picks
    // which candidate is played. Every StartGame (live, replayed, or late
    // join) reseeds to the same constant so replayed AI picks match the
    // original live trajectory.
    *sinks.bot_driver = crate::bot_player::BotDriver::default();
    *sinks.game_state = GameState::new(scenario);
    sinks
        .game_state
        .optional_rules
        .extend_from_slice(optional_rules);
    let map_kind = crate::scenario_setup::map_kind_for_scenario(scenario);
    sinks.game_state.board = std::sync::Arc::new(BoardInfo::from_map_data(
        sinks.loaded_annotations.map(map_kind),
    ));
    sinks.pending_map_load.0 = Some(map_kind);
    // Movement routes and the press belong to the previous game.
    sinks.unit_paths.0.clear();
    *sinks.press = crate::telegram::TelegramLog::default();
    map_kind
}

/// The rules identity of a counter sprite (deterministic: each physical
/// counter maps to exactly one `UnitId`, so every peer resolves the same one).
fn unit_for_sprite(sprite: &SpriteRef) -> Option<UnitId> {
    omdurman_rules::unit_id_for_section_pos(sprite.section_name, sprite.col as u8, sprite.row as u8)
}

/// Translate a sprite-shaped `GameEvent` (`PlaceUnit` / `MoveUnit` /
/// `RemoveUnit`) into the engine effect it stands for, in the context of the
/// engine state it is about to be applied to. `None` for events that carry no
/// resolvable counter (or for non-sprite variants).
///
/// * `PlaceUnit` deploys during Setup (§9.2/§9.3) and enters as a
///   *reinforcement* during a Movement phase (§9.112/§9.113 Campaign order of
///   appearance; §9.322 FoK turn-1 edge) — `DeployUnit` is Setup-only.
/// * `MoveUnit` is the engine `MoveUnit` (allowance, ZOC, night-halving are
///   validated by the engine).
/// * `RemoveUnit` is the Setup pickup (`RemoveDeployedUnit`), acted by the
///   counter's owner.
pub(crate) fn sprite_event_effect(event: &GameEvent, gs: &GameState) -> Option<GameEffect> {
    match event {
        GameEvent::PlaceUnit { sprite, coord, .. } => {
            let id = unit_for_sprite(sprite)?;
            let profile = omdurman_rules::unit_profiles::profile_for_unit(id)?;
            let placement = UnitPlacement {
                id,
                position: *coord,
                profile,
                state: UnitState::default(),
            };
            Some(if matches!(gs.phase, Phase::Movement) {
                GameEffect::PlaceReinforcements(vec![placement])
            } else {
                GameEffect::DeployUnit(placement)
            })
        }
        GameEvent::MoveUnit {
            sprite,
            to_q,
            to_r,
            cost,
            path,
        } => Some(GameEffect::MoveUnit {
            unit_id: unit_for_sprite(sprite)?,
            to: HexCoord::new(*to_q, *to_r),
            cost: *cost,
            path: path.clone(),
        }),
        GameEvent::RemoveUnit { sprite } => Some(GameEffect::RemoveDeployedUnit {
            unit_id: unit_for_sprite(sprite)?,
            player: omdurman_rules::unit_profiles::section_owner(sprite.section_name)?,
        }),
        GameEvent::StartGame { .. }
        | GameEvent::Effect(_)
        | GameEvent::SeatAssigned { .. }
        | GameEvent::SeatCarved { .. }
        | GameEvent::Telegram { .. }
        | GameEvent::Gazette { .. } => None,
    }
}

/// Extend a unit's turn path with an accepted move. `path` is the sequence of
/// hexes *entered* this move (ending at `to`); when it is empty (legacy record)
/// we fall back to a single hop straight to `to`.
fn record_move_path(
    paths: &mut UnitPaths,
    unit_id: UnitId,
    from: HexCoord,
    path: &[HexCoord],
    to: HexCoord,
) {
    let mut prev = from;
    let steps: &[HexCoord] = if path.is_empty() { &[to] } else { path };
    for &step in steps {
        if step != prev {
            paths.record_step(unit_id, prev, step);
            prev = step;
        }
    }
}

/// Apply one engine effect, recording the route of an accepted move and
/// resetting the routes when the turn passes to the other player. Returns
/// whether the engine accepted it.
fn apply_engine_effect(effect: &GameEffect, sinks: &mut EventSinks<'_>) -> bool {
    let active_before = sinks.game_state.active_player;
    let move_from = match effect {
        GameEffect::MoveUnit { unit_id, .. } => sinks
            .game_state
            .find_unit(*unit_id)
            .map(|u| (*unit_id, u.position)),
        _ => None,
    };
    debug!(?effect, "applying game effect");
    if let Err(error) = apply_effect(sinks.game_state, effect) {
        warn!(%error, ?effect, "effect rejected by rules engine");
        return false;
    }
    debug!(
        phase = ?sinks.game_state.phase,
        turn = sinks.game_state.current_turn.value(),
        active_player = ?sinks.game_state.active_player,
        "effect applied"
    );
    if sinks.game_state.active_player != active_before {
        // The arrows show the moves made *this* player-turn.
        sinks.unit_paths.0.clear();
    }
    if let (GameEffect::MoveUnit { to, path, .. }, Some((uid, from))) = (effect, move_from) {
        record_move_path(sinks.unit_paths, uid, from, path, *to);
    }
    true
}

/// Apply one recorded `GameEvent` to the engine. The only application path:
/// the live echo and the replay both call this, in sequence order, so the
/// engine state is a pure function of the log. Returns whether the event was
/// accepted (a rejected event is logged and leaves the state untouched).
///
/// The caller drains the engine's observations afterwards (live: into the UI
/// queue; rebuild: discarded).
pub(crate) fn apply_game_event(event: &GameEvent, sinks: &mut EventSinks<'_>) -> bool {
    match event {
        GameEvent::StartGame {
            seats,
            scenario,
            optional_rules,
        } => {
            apply_start_game(
                StartGameFields {
                    seats,
                    scenario: *scenario,
                    optional_rules,
                },
                sinks,
            );
            true
        }
        GameEvent::SeatAssigned {
            seat,
            previous,
            holder,
        } => {
            let ok = crate::seats::assign_seat(&mut sinks.seats.0, *seat, *previous, *holder);
            if !ok {
                warn!(?event, "seat assignment no longer applies; ignored");
            }
            ok
        }
        GameEvent::SeatCarved {
            faction,
            scope,
            holder,
        } => {
            let ok = crate::seats::carve_seat(&mut sinks.seats.0, *faction, scope, *holder);
            if !ok {
                warn!(?event, "sub-faction takeover no longer applies; ignored");
            }
            ok
        }
        GameEvent::Effect(effect) => apply_engine_effect(effect, sinks),
        // Presentation only: the host's telegram / Gazette text, filed the
        // same on every peer (the first for a turn wins).
        GameEvent::Telegram { .. } | GameEvent::Gazette { .. } => sinks.press.file(event),
        GameEvent::PlaceUnit { .. } | GameEvent::MoveUnit { .. } | GameEvent::RemoveUnit { .. } => {
            let Some(effect) = sprite_event_effect(event, sinks.game_state) else {
                warn!(?event, "sprite event names no rules counter; ignored");
                return false;
            };
            apply_engine_effect(&effect, sinks)
        }
    }
}

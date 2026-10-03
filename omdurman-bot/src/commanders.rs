//! The two historical commanders: **Kitchener** (Anglo-Egyptian) and
//! **Khalifa** (Dervish).
//!
//! Unlike [`crate::aggressive`] (a single swarm doctrine for either side),
//! each commander plays *his* side's historical plan, adapted to the scenario
//! in play. The doctrine is distilled from `docs/strategy/*.md` (which cites
//! the manual sections); scoring is a one-decision API
//! ([`Commander::pick`]) over the legal-action candidate list so the same
//! code drives both the headless playthroughs and the app's in-game AI
//! commanders (see `omdurman-app/src/bot_player.rs`).
//!
//! # Kitchener — "firepower first, then the Tomb"
//!
//! Massed fire over piecemeal shots (§6.14), brigade integrity (§5.54),
//! Maxims fire twice (§6.42), counter-battery against the enemy artillery that
//! alone can breach walls, sink gunboats, or destroy forts (§6.61-§6.63);
//! defensively: hold the wall/gate line and the palace ring in Fall of
//! Khartoum, make every Dervish assault pay (each kill downgrades their
//! victory level, §9.35), keep leaders bodyguarded (§6.51); cavalry
//! retreats from hopeless melee (§7.5); take advances only into ground that
//! is not a death trap (§6.82). On the campaign map the axis is the Mahdi's
//! Tomb (25 VP, §9.14) behind a formed line.
//!
//! # Khalifa — "the clock is the weapon"
//!
//! Fall of Khartoum is a race: kill GORDON by turn 4/5/6 or lose (§9.35), so
//! the assault closes under night cover from turn 1 (§9.341, §8.1), masses by
//! tribe at one wall, breaches it with artillery (§6.63), storms through with
//! melee (Dervish +2, §7.7), and pours through every mandatory advance
//! (§7.6). Losses matter (§9.35) — no suicide melees into unweakened stacks —
//! but speed outranks blood. On the campaign map: swarm in waves, screen with
//! ZOC crusts (§5.43), guard the Khalifa (10 VP, §9.14), feed each
//! reinforcement wave straight into the line (§9.112).

use omdurman_rules::effects::{GameEffect, GameState};
use omdurman_rules::{Phase, UnitId, UnitIdentity};
use omdurman_types::{DayNight, HexCoord, HexsideKind, Location, Player, Scenario};

use crate::rng::BotRng;

/// The commander personality, one per faction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Commander {
    /// Anglo-Egyptian: deliberate, firepower-first (Horatio Kitchener).
    Kitchener,
    /// Dervish: the all-out assault, speed over blood (Abdallahi, the Khalifa).
    Khalifa,
}

impl Commander {
    /// The historical commander of `player`'s faction.
    pub fn for_player(player: Player) -> Self {
        match player {
            Player::AngloEgyptian => Self::Kitchener,
            Player::Dervish => Self::Khalifa,
        }
    }

    /// Display name for logs and the lobby UI.
    pub fn name(self) -> &'static str {
        match self {
            Self::Kitchener => "Kitchener",
            Self::Khalifa => "Khalifa",
        }
    }

    /// Pick the best-scoring legal action (ties broken by `rng` so runs stay
    /// seed-reproducible). `player` is the acting side — the phase's candidate
    /// owner, which differs from `state.active_player` in defensive fire (§6.7).
    pub fn pick(
        &self,
        state: &GameState,
        player: Player,
        candidates: &[GameEffect],
        rng: &mut BotRng,
    ) -> GameEffect {
        let planned;
        let candidates = if matches!(
            state.phase,
            Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
        ) {
            planned = crate::fire_plan::plan_fire(state, candidates);
            if planned.is_empty() {
                return GameEffect::AdvancePhase;
            }
            &planned[..]
        } else {
            candidates
        };
        let mut best = i32::MIN;
        let mut best_idxs: Vec<usize> = Vec::new();
        for (i, effect) in candidates.iter().enumerate() {
            let score = self.score(effect, state, player);
            if score > best {
                best = score;
                best_idxs.clear();
                best_idxs.push(i);
            } else if score == best {
                best_idxs.push(i);
            }
        }
        let idx = rng.choose(&best_idxs).copied().unwrap_or(0);
        candidates[idx].clone()
    }

    /// Score one candidate. Higher = more in keeping with the commander's
    /// doctrine.
    fn score(&self, effect: &GameEffect, state: &GameState, player: Player) -> i32 {
        match self {
            Self::Kitchener => kitchener_score(effect, state, player),
            Self::Khalifa => khalifa_score(effect, state, player),
        }
    }
}

/// Setup-phase pick for commander-driven play: the Setup candidates mix both
/// sides' deployments, and each commander must arrange **its own** force by
/// its own doctrine — never the enemy's garrison. Every candidate is scored
/// by the commander of the side that owns it; confirming ready and advancing
/// stay last-resort moves.
pub fn pick_setup(
    state: &GameState,
    candidates: &[GameEffect],
    _agents: &crate::agent::Agents,
    rng: &mut BotRng,
) -> GameEffect {
    let mut best = i32::MIN;
    let mut best_idxs: Vec<usize> = Vec::new();
    for (i, effect) in candidates.iter().enumerate() {
        let score = match effect {
            GameEffect::DeployUnit(p) => {
                let owner = p.profile.identity.owner();
                Commander::for_player(owner).score(effect, state, owner)
            }
            GameEffect::RemoveDeployedUnit { player, .. }
            | GameEffect::ConfirmSetupReady { player, .. } => {
                Commander::for_player(*player).score(effect, state, *player)
            }
            GameEffect::AdvancePhase => 1,
            _ => 0,
        };
        if score > best {
            best = score;
            best_idxs.clear();
            best_idxs.push(i);
        } else if score == best {
            best_idxs.push(i);
        }
    }
    let idx = rng.choose(&best_idxs).copied().unwrap_or(0);
    candidates[idx].clone()
}

/// Pick the best-scoring candidate that the engine *actually accepts*,
/// validated on a cloned state before returning. The bot's action
/// enumerator can be weaker than `apply_effect` (e.g. §5.52 tribe stacking);
/// an in-game commander must never submit an effect that would be rejected.
/// Returns `GameEffect::AdvancePhase` when nothing legal remains.
///
/// The ranking is deterministic (score, then enumeration order) and draws
/// nothing from `_rng`; the parameter is kept so callers need not change if
/// rng tie-breaking is added later. `memory` is the driver's
/// [`crate::move_memory::MoveMemory`], fed with the pick.
pub fn pick_validated(
    state: &GameState,
    player: Player,
    candidates: &[GameEffect],
    _rng: &mut BotRng,
    memory: &mut crate::move_memory::MoveMemory,
) -> GameEffect {
    // Fire phases: merge the per-stack shots into planned combined attacks
    // (§6.14) before ranking -- see `crate::fire_plan`. Movement: a unit
    // steps only onto a hex worth more to it ("position values" below), and
    // never back onto ground it covered this phase (`crate::move_memory`).
    let planned;
    let candidates = if matches!(
        state.phase,
        Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
    ) {
        planned = crate::fire_plan::plan_fire(state, candidates);
        &planned[..]
    } else {
        candidates
    };
    let ranked = if state.phase == Phase::Movement {
        rank_planned(state, player, candidates, memory)
    } else {
        rank(state, player, candidates)
    };
    for candidate in ranked {
        if memory.revisits(state, &candidate) {
            continue;
        }
        let mut test = state.clone();
        let res = omdurman_rules::effects::apply_effect(&mut test, &candidate);
        if res.is_err()
            && let GameEffect::MoveUnit { unit_id, .. } = &candidate
        {
            // The planned step is refused (a hex the plan could not foresee
            // -- walled-city entry, a stack filled meanwhile): hold.
            memory.abandon_plan(state, *unit_id);
        }
        if res.is_ok() {
            memory.record(state, &candidate);
            return candidate;
        }
    }
    GameEffect::AdvancePhase
}

/// Setup-phase counterpart of [`pick_validated`]: score by owner (each
/// commander arranges its own force), validate on a clone. `own_side`, when
/// `Some`, restricts the candidates to that side's actions (used when only
/// one faction is AI-commanded; the human deploys their own units).
/// Like [`pick_validated`], deterministic: `_rng` is not drawn from.
pub fn pick_setup_validated(
    state: &GameState,
    candidates: &[GameEffect],
    own_side: Option<Player>,
    _rng: &mut BotRng,
) -> GameEffect {
    let owned: Vec<GameEffect> = candidates
        .iter()
        .filter(|e| match e {
            GameEffect::DeployUnit(p) => {
                own_side.is_none_or(|side| p.profile.identity.owner() == side)
            }
            GameEffect::RemoveDeployedUnit { player, .. }
            | GameEffect::ConfirmSetupReady { player, .. } => {
                own_side.is_none_or(|side| *player == side)
            }
            _ => true,
        })
        .cloned()
        .collect();
    for candidate in rank_setup(state, &owned) {
        let mut test = state.clone();
        if omdurman_rules::effects::apply_effect(&mut test, &candidate).is_ok() {
            return candidate;
        }
    }
    GameEffect::AdvancePhase
}

/// The Khalifa's choice of deserters (§8.2: "of the Dervish player's
/// choosing"; the Khalifa, artillery, gunboats and forts are exempt). The
/// roll and the count stay as rolled; only *who* goes changes: a deserter
/// scores the Anglo-Egyptians nothing (§8.2), so shed first the disrupted,
/// then the units standing in the British fire lanes and not in contact --
/// the ones Kitchener would otherwise have killed for a VP each -- and never
/// a leader or the Tomb garrison while anyone else can go. Any other effect
/// passes through unchanged.
pub fn choose_deserters(state: &GameState, effect: GameEffect) -> GameEffect {
    let GameEffect::DervishDesertion { roll, deserters } = effect else {
        return effect;
    };
    let count = deserters.len();
    let enemy = Player::AngloEgyptian;
    let tomb = state.board.hex_of_location(Location::MahdisTomb);
    let mut pool: Vec<(i64, UnitId)> = state
        .units
        .iter()
        .filter(|u| {
            u.profile.identity.owner() == Player::Dervish
                && !u.profile.identity.is_desertion_exempt()
        })
        .map(|u| {
            let in_contact = u.position.neighbors().iter().any(|&n| {
                state
                    .units_in_hex(n)
                    .iter()
                    .any(|e| e.profile.identity.owner() == enemy)
            });
            let lanes = crate::threat::fire_reaching(state, u.position, enemy) as i64;
            let keep = matches!(u.profile.identity, UnitIdentity::DervishLeader(_))
                || Some(u.position) == tomb;
            let rank = i64::from(u.state.disrupted) * 1000 + lanes * 10
                - i64::from(in_contact) * 500
                - i64::from(keep) * 100_000
                - i64::from(u.profile.melee.map(|m| m.value()).unwrap_or(0));
            (rank, u.id)
        })
        .collect();
    // Stable: equal ranks keep board order, so the pick is reproducible.
    pool.sort_by_key(|(rank, _)| std::cmp::Reverse(*rank));
    GameEffect::DervishDesertion {
        roll,
        deserters: pool.into_iter().take(count).map(|(_, id)| id).collect(),
    }
}

/// Movement-phase ranking: each unit's only move is the next step of its
/// plan for the phase ([`plan_move`]), ranked by what the plan gains; every
/// other candidate (reinforcements, demolition, the desertion roll, ending
/// the phase) keeps its commander score. A unit whose plan is to hold
/// offers no move, so ending the phase follows once every plan is walked.
fn rank_planned(
    state: &GameState,
    player: Player,
    candidates: &[GameEffect],
    memory: &mut crate::move_memory::MoveMemory,
) -> Vec<GameEffect> {
    let commander = Commander::for_player(player);
    let mut scored: Vec<(i32, usize)> = Vec::new();
    for (i, effect) in candidates.iter().enumerate() {
        let score = match effect {
            GameEffect::MoveUnit { unit_id, to, .. } => {
                let Some(unit) = state.find_unit(*unit_id) else {
                    continue;
                };
                let (next, gain) =
                    memory.plan_for(state, *unit_id, || plan_move(state, unit, player));
                if next != Some(*to) {
                    continue;
                }
                10 + gain.clamp(1, 50)
            }
            _ => commander.score(effect, state, player),
        };
        scored.push((score, i));
    }
    scored.sort_by_key(|(s, _)| std::cmp::Reverse(*s));
    scored
        .into_iter()
        .map(|(_, i)| candidates[i].clone())
        .collect()
}

/// `player`'s goal for one unit (its commander's).
fn goal_for(state: &GameState, unit_id: UnitId, player: Player) -> Option<Goal> {
    match Commander::for_player(player) {
        Commander::Kitchener => kitchener_goal(state, unit_id, player),
        Commander::Khalifa => khalifa_goal(state, unit_id, player),
    }
}

/// `player`'s value of `unit` standing on `at` (its commander's).
fn position_value(
    state: &GameState,
    unit: &omdurman_rules::UnitPlacement,
    at: HexCoord,
    goal: Option<Goal>,
    player: Player,
) -> i32 {
    let palace = palace_hex(state);
    match Commander::for_player(player) {
        Commander::Kitchener => kitchener_position_value(state, unit, at, goal, player, palace),
        Commander::Khalifa => {
            khalifa_position_value(state, unit, at, goal, player, palace, is_night(state))
        }
    }
}

/// A unit's plan for the movement phase: the hex worth most to it among
/// those it can reach with the movement points it has left (§5.11-§5.13),
/// and the cheapest way there -- or, when nothing it can reach beats where
/// it stands, no move at all. Returns the hexes to enter and the gain.
///
/// The reach is the engine's step costs (terrain, roads, hexside
/// surcharges) over hexes free of the enemy, ending at the first enemy ZOC
/// (§5.43); a gunboat keeps to the Nile and passes no other gunboat. The
/// end must stack legally (§5.5). Steps the search cannot foresee (the
/// walled city's entry rules, §5.23) are refused when taken, and the unit
/// then holds.
fn plan_move(
    state: &GameState,
    unit: &omdurman_rules::UnitPlacement,
    player: Player,
) -> (Vec<HexCoord>, i32) {
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, HashMap};
    let budget = i32::from(state.remaining_movement(unit.id));
    if budget <= 0 {
        return (Vec::new(), 0);
    }
    let goal = goal_for(state, unit.id, player);
    let value = |at: HexCoord| position_value(state, unit, at, goal, player);
    let enemy = player.opponent();
    let is_boat = unit.profile.kind.is_boat();
    let start = unit.position;
    let mut cost: HashMap<HexCoord, i32> = HashMap::from([(start, 0)]);
    let mut prev: HashMap<HexCoord, HexCoord> = HashMap::new();
    let mut heap = BinaryHeap::from([Reverse((0i32, start.q, start.r))]);
    while let Some(Reverse((c, q, r))) = heap.pop() {
        let hex = HexCoord::new(q, r);
        if cost.get(&hex).is_some_and(|&best| c > best) {
            continue;
        }
        // Entering an enemy ZOC ends the move (§5.26, §5.43).
        if hex != start && !is_boat && state.hex_in_enemy_zoc(hex, player, unit.profile.kind) {
            continue;
        }
        for n in hex.neighbors() {
            let occupied = state.units_in_hex(n);
            if occupied.iter().any(|u| u.profile.identity.owner() == enemy) {
                continue;
            }
            let step = if is_boat {
                if !state.board.is_nile(n) || occupied.iter().any(|u| u.profile.kind.is_boat()) {
                    continue;
                }
                1
            } else {
                let mut there = *unit;
                there.position = hex;
                match state.movement_cost_for(&there, &[n]) {
                    Some(mp) => i32::from(mp.value()),
                    None => continue,
                }
            };
            let nc = c + step;
            if nc > budget || cost.get(&n).is_some_and(|&best| nc >= best) {
                continue;
            }
            cost.insert(n, nc);
            prev.insert(n, hex);
            heap.push(Reverse((nc, n.q, n.r)));
        }
    }
    let here = value(start);
    // The best end: most value, then the cheaper way (ties keep the board
    // order of the sorted hexes, so the plan is reproducible).
    let mut ends: Vec<(HexCoord, i32)> = cost.into_iter().filter(|(h, _)| *h != start).collect();
    ends.sort_by_key(|(h, c)| (*c, h.q, h.r));
    let mut best: Option<(HexCoord, i32)> = None;
    for (hex, _) in ends {
        if state.check_stacking(unit, hex).is_err() {
            continue;
        }
        let v = value(hex);
        if v > here && best.is_none_or(|(_, b)| v > b) {
            best = Some((hex, v));
        }
    }
    let Some((end, v)) = best else {
        return (Vec::new(), 0);
    };
    let mut way = vec![end];
    while let Some(&p) = prev.get(way.last().expect("non-empty")) {
        if p == start {
            break;
        }
        way.push(p);
    }
    way.reverse();
    (way, v - here)
}

/// Candidates ordered best-first by the commander's score. The sort is
/// stable, so equal scores keep their enumeration order -- no rng is drawn,
/// and the order is reproducible from the state alone.
fn rank(state: &GameState, player: Player, candidates: &[GameEffect]) -> Vec<GameEffect> {
    let commander = Commander::for_player(player);
    let mut scored: Vec<(i32, usize)> = candidates
        .iter()
        .enumerate()
        .map(|(i, e)| (commander.score(e, state, player), i))
        .collect();
    scored.sort_by_key(|(s, _)| std::cmp::Reverse(*s));
    scored
        .into_iter()
        .map(|(_, i)| candidates[i].clone())
        .collect()
}

/// Setup candidates ordered best-first per owning side's commander (stable:
/// ties keep enumeration order; no rng involved).
fn rank_setup(state: &GameState, candidates: &[GameEffect]) -> Vec<GameEffect> {
    let mut scored: Vec<(i32, usize)> = candidates
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let s = match e {
                GameEffect::DeployUnit(p) => {
                    let owner = p.profile.identity.owner();
                    Commander::for_player(owner).score(e, state, owner)
                }
                GameEffect::RemoveDeployedUnit { player, .. }
                | GameEffect::ConfirmSetupReady { player, .. } => {
                    Commander::for_player(*player).score(e, state, *player)
                }
                GameEffect::AdvancePhase => 1,
                _ => 0,
            };
            (s, i)
        })
        .collect();
    scored.sort_by_key(|(s, _)| std::cmp::Reverse(*s));
    scored
        .into_iter()
        .map(|(_, i)| candidates[i].clone())
        .collect()
}

// ---------------------------------------------------------------------------
// Shared board geometry helpers
// ---------------------------------------------------------------------------

/// The Palace hex in Fall of Khartoum (GORDON's fixed post, §9.346).
fn palace_hex(state: &GameState) -> Option<HexCoord> {
    (state.scenario == Scenario::FallOfKhartoum)
        .then(|| state.board.hex_of_location(Location::Palace))
        .flatten()
}

/// Whether it is night (§8.1): AE movement and all fire ranges halved,
/// no howitzer.
fn is_night(state: &GameState) -> bool {
    state.day_night == DayNight::Night
}

/// Total printed fire factors of `player`'s units in `hex`.
fn fire_strength_in(state: &GameState, hex: HexCoord, player: Player) -> i32 {
    state
        .units_in_hex(hex)
        .into_iter()
        .filter(|u| u.profile.identity.owner() == player)
        .map(|u| u.profile.fire.map(|f| f.value()).unwrap_or(0) as i32)
        .sum()
}

/// Number of `player`'s combat units in `hex` (leaders and forts excluded —
/// they defend but are not rifles).
fn combat_count_in(state: &GameState, hex: HexCoord, player: Player) -> i32 {
    state
        .units_in_hex(hex)
        .into_iter()
        .filter(|u| {
            u.profile.identity.owner() == player
                && !matches!(
                    u.profile.identity,
                    UnitIdentity::AngloEgyptianLeader(_)
                        | UnitIdentity::DervishLeader(_)
                        | UnitIdentity::DervishFort
                        | UnitIdentity::AngloEgyptianFort
                )
        })
        .count() as i32
}

/// Summed fire factors of all of `opponent`'s units adjacent to `hex` — the
/// defensive fire a unit stepping into `hex` should expect (§6.7). Halved at
/// night (§8.1).
fn adjacent_enemy_fire(state: &GameState, hex: HexCoord, player: Player) -> i32 {
    let opponent = player.opponent();
    let raw: i32 = hex
        .neighbors()
        .iter()
        .map(|&n| fire_strength_in(state, n, opponent))
        .sum();
    if is_night(state) { raw / 2 } else { raw }
}

/// Count of `opponent`'s unit-stacks adjacent to `hex` (counterattack /
/// supporting-fire exposure).
fn adjacent_enemy_stacks(state: &GameState, hex: HexCoord, player: Player) -> i32 {
    let opponent = player.opponent();
    hex.neighbors()
        .iter()
        .filter(|&&n| !state.units_in_hex(n).is_empty())
        .map(|&n| {
            state
                .units_in_hex(n)
                .into_iter()
                .filter(|u| u.profile.identity.owner() == opponent)
                .count()
        })
        .sum::<usize>() as i32
}

/// Movement progress of `unit` stepping `to` toward `goal` (positive = closer).
fn progress(state: &GameState, unit_id: UnitId, to: HexCoord, goal: Option<HexCoord>) -> i32 {
    let (Some(goal), Some(unit)) = (goal, state.find_unit(unit_id)) else {
        return 0;
    };
    unit.position.distance(goal) as i32 - to.distance(goal) as i32
}

/// The Khalifa's single assault axis — **static**, derived from board
/// geometry only (the south-east §9.322 entry corner + the palace), never
/// from live unit positions: a live centroid moves with the wave, flips the
/// axis mid-phase, and the assault mills around instead of converging.
/// One breach, one wave, one corridor — concentration is everything against
/// a walled city (§9.322, §6.63).
pub fn assault_axis_wall(state: &GameState) -> Option<HexCoord> {
    let palace = palace_hex(state)?;
    // The south-east corner of the diamond board: the playable hex with the
    // largest q+r (east = no hex at q+1, south = no hex at r+1).
    let corner = state
        .board
        .terrain
        .keys()
        .copied()
        .max_by_key(|h| h.q + h.r)?;
    let mut best: Option<(i32, HexCoord)> = None;
    for (hr, kind) in &state.board.hexsides {
        if *kind != HexsideKind::Wall || state.wall_is_breached(hr.a, hr.b) {
            continue;
        }
        for &end in &[hr.a, hr.b] {
            let key = corner.distance(end) as i32 * 2 + palace.distance(end) as i32;
            if best.is_none_or(|(bk, _)| key < bk) {
                best = Some((key, end));
            }
        }
    }
    best.map(|(_, hex)| hex)
}

/// The outside (Dervish-side) hex of the gate or breach hexside closest to
/// the assault axis: the corridor the wave files through (§5.23, §7.2).
/// `None` while no gate/breach lies near the axis (then the wave masses on
/// the wall and waits for the guns).
pub fn assault_corridor(state: &GameState) -> Option<HexCoord> {
    let palace = palace_hex(state)?;
    let axis = assault_axis_wall(state)?;
    let mut best: Option<(i32, HexCoord)> = None;
    for (hr, _) in &state.board.hexsides {
        if !matches!(
            state.hexside_effective(hr.a, hr.b),
            Some(HexsideKind::Gate | HexsideKind::Breach)
        ) {
            continue;
        }
        let near = axis.distance(hr.a).min(axis.distance(hr.b));
        if near > 3 {
            continue;
        }
        // The outside endpoint is the one farther from the palace.
        let outside = if palace.distance(hr.a) >= palace.distance(hr.b) {
            hr.a
        } else {
            hr.b
        };
        let key = near as i32;
        if best.is_none_or(|(bk, _)| key < bk) {
            best = Some((key, outside));
        }
    }
    best.map(|(_, hex)| hex)
}

/// During Setup, the distance from `hex` to the nearest south/east map edge —
/// the §9.322 Dervish entry zone — used as the threat anchor when the enemy
/// is not yet on the board.
fn dist_to_dervish_entry(state: &GameState, hex: HexCoord) -> i32 {
    let mut best = i32::MAX;
    for &h in state.board.terrain.keys() {
        let on_south = !state
            .board
            .terrain
            .contains_key(&HexCoord::new(h.q, h.r + 1));
        let on_east = !state
            .board
            .terrain
            .contains_key(&HexCoord::new(h.q + 1, h.r));
        if on_south || on_east {
            best = best.min(hex.distance(h) as i32);
        }
    }
    best
}

/// Whether `hex` touches a standing wall or gate hexside (the city line,
/// §5.23). Read through the engine's effective hexside so a §6.63 breach no
/// longer counts as wall.
fn on_city_line(state: &GameState, hex: HexCoord) -> bool {
    hex.neighbors().iter().any(|&n| {
        matches!(
            state.hexside_effective(hex, n),
            Some(HexsideKind::Wall) | Some(HexsideKind::Gate)
        )
    })
}

/// Which side of the corridor network `hex` sits on, for the FoK defence:
/// - `Inside`: the hex itself is a gate/breach plug (it touches the corridor
///   hexside on the palace side);
/// - `Staging`: a normal neighbour of an inside plug, on the palace side —
///   the re-plug reserve;
/// - `Outside`: across a corridor from the palace side — a trap for a
///   garrison (no wall at its back, melee-reached by the whole horde).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Inside,
    Staging,
    Outside,
}

fn corridor_side(state: &GameState, hex: HexCoord, palace: Option<HexCoord>) -> Option<Side> {
    let palace = palace?;
    // Depth-1 "inside plug" test as a local closure — never recurse here:
    // a recursive staging check branches exponentially and blows the stack.
    let corridor = |a: HexCoord, b: HexCoord| {
        matches!(
            state.hexside_effective(a, b),
            Some(HexsideKind::Gate) | Some(HexsideKind::Breach)
        )
    };
    let is_inside = |h: HexCoord| -> bool {
        h.neighbors()
            .iter()
            .any(|&n| corridor(h, n) && palace.distance(h) <= palace.distance(n))
    };
    for &n in hex.neighbors().iter() {
        if corridor(hex, n) {
            return if palace.distance(hex) <= palace.distance(n) {
                Some(Side::Inside)
            } else {
                Some(Side::Outside)
            };
        }
    }
    // Staging: adjacent to an inside plug through an ordinary hexside.
    for &n in hex.neighbors().iter() {
        if is_inside(n) && palace.distance(hex) <= palace.distance(n) {
            return Some(Side::Staging);
        }
    }
    None
}

/// Whether this placement is one of the scenario-fixed units (GORDON in the
/// palace, the North Fort, §9.321/§9.344): they must come off the list
/// immediately, before any free deployment can fill their hex.
fn is_fixed_placement(p: &omdurman_rules::UnitPlacement) -> bool {
    matches!(
        p.profile.identity,
        UnitIdentity::DervishFort
            | UnitIdentity::AngloEgyptianFort
            | UnitIdentity::AngloEgyptianLeader(omdurman_rules::BritishLeader::Gordon)
    )
}

// ---------------------------------------------------------------------------
// Movement: position values
// ---------------------------------------------------------------------------
//
// A unit moves only to stand somewhere better. Each commander values a hex
// for a unit -- the goal, the fire it would stand in, the company it would
// keep -- as a function of that hex alone, and a step scores the gain over
// the hex the unit stands on. A unit with no gaining step stays where it is
// (ending the phase outranks every losing step), and a unit that has gained
// cannot gain by stepping back: the value only climbs, so the walk never
// returns onto ground it left while the board stands still.

/// Where a unit wants to stand (see [`Goal::term`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Goal {
    /// On this hex.
    Hex(HexCoord),
    /// `standoff` hexes from `target`: closing from farther, backing off
    /// (more gently) from nearer -- the gun line, the rifle line.
    Band { target: HexCoord, standoff: u32 },
    /// On this hex by the movement-point road: walls closed but at gates and
    /// breaches (§5.23), terrain costs (§5.11).
    March(HexCoord),
}

impl Goal {
    /// The goal's part of a hex's value: `per_hex` for every hex still to
    /// go. A land unit counts the movement-point road (§5.11 terrain, walls
    /// open only at gates and breaches §5.23, the Zariba only at its ends
    /// §9.233): walking round an obstacle toward its gap is progress, where
    /// the straight-line distance would see none and the unit would never
    /// find the way out. Boats count the river (see
    /// [`crate::threat::river_cost`]).
    fn term(self, state: &GameState, at: HexCoord, per_hex: i32, land: bool) -> i32 {
        // Distances in quarter hexes: a boat's river way is (see
        // `river_cost`), and a land road is scaled to match.
        let dist4 = |goal: HexCoord| {
            if land {
                4 * crate::threat::path_cost(state, at, goal).unwrap_or(100)
            } else {
                crate::threat::river_cost(state, at, goal).unwrap_or(4 * at.distance(goal) as i32)
            }
        };
        match self {
            Goal::Hex(goal) => -per_hex * dist4(goal) / 4,
            Goal::Band { target, standoff } => {
                let d = dist4(target);
                let s = 4 * standoff as i32;
                if d >= s {
                    -per_hex * (d - s) / 4
                } else {
                    -per_hex * (s - d) / 8
                }
            }
            Goal::March(goal) => match crate::threat::path_cost(state, at, goal) {
                Some(cost) => -per_hex * cost,
                // Cut off from the goal: worse than any reachable hex.
                None => -per_hex * 100,
            },
        }
    }
}

/// A step's rank from its gain: a gaining step outranks ending the phase
/// (score 1), a losing or level one never does -- the unit holds.
fn move_score(gain: i32) -> i32 {
    if gain > 0 { 10 + gain } else { gain - 10 }
}

/// `player`'s combat units on `hex` other than `unit` itself (the company a
/// unit would keep there).
fn friends_in(state: &GameState, hex: HexCoord, player: Player, unit: UnitId) -> i32 {
    state
        .units_in_hex(hex)
        .into_iter()
        .filter(|u| {
            u.id != unit
                && u.profile.identity.owner() == player
                && !u.profile.kind.is_boat()
                && !matches!(
                    u.profile.identity,
                    UnitIdentity::AngloEgyptianLeader(_)
                        | UnitIdentity::DervishLeader(_)
                        | UnitIdentity::DervishFort
                        | UnitIdentity::AngloEgyptianFort
                )
        })
        .count() as i32
}

/// What a hex of progress to the goal is worth against the fire and company
/// terms of the position values, for each commander.
const KITCHENER_PER_HEX: i32 = 24;
const KHALIFA_PER_HEX: i32 = 26;
/// What a point of effective fire factor in reach is worth to Kitchener
/// (see [`crate::threat::best_shot_from`]): a good firing position is worth
/// about a hex of progress.
/// Score per unit of enemy melee strength that can reach a hex next turn
/// (`threat::melee_reaching`): non-leaders keep clear of the spears.
const KITCHENER_SPEAR_FEAR: f32 = 0.08;
const KITCHENER_SHOT_WEIGHT: f32 = 2.0;
/// How much the Anglo-Egyptian line fears the Dervish fire lanes off the FoK
/// walls, over the per-unit weights in [`kitchener_position_value`] (set by
/// the arena: standing in reach against shooting from it).
const LANE_FEAR: f32 = 3.0;

/// Kitchener's value of `unit` standing on `at`: progress to its goal, minus
/// the fire it would stand in, plus the company it would keep.
fn kitchener_position_value(
    state: &GameState,
    unit: &omdurman_rules::UnitPlacement,
    at: HexCoord,
    goal: Option<Goal>,
    player: Player,
    palace: Option<HexCoord>,
) -> i32 {
    let land = !unit.profile.kind.is_boat();
    let mut value = goal.map_or(0, |g| g.term(state, at, KITCHENER_PER_HEX, land));
    if !land {
        // Only artillery may fire at a gunboat, and only a 3 or more on
        // the table sinks it (§6.61): the rifles and spears a land unit
        // fears are nothing to a boat. It hunts by its shot and keeps out
        // of the guns' reach -- forts and batteries (a boat is 10 VP to
        // the Dervish in the Campaign, §9.14).
        let guns = crate::threat::artillery_reaching(state, at, player.opponent());
        return value
            + (KITCHENER_SHOT_WEIGHT * crate::threat::best_shot_from(state, unit, at)) as i32
            - guns as i32;
    }
    // Don't stand in a killing zone (§6.7): leaders and artillery fear
    // defensive fire most; at night the fire is halved (§8.1).
    let is_leader = matches!(unit.profile.identity, UnitIdentity::AngloEgyptianLeader(_));
    let is_fragile = is_leader || unit.profile.identity == UnitIdentity::AngloEgyptianArtillery;
    let exposure = adjacent_enemy_fire(state, at, player);
    value -= if is_fragile {
        exposure / 3
    } else {
        exposure / 6
    };
    // Off the FoK walls: keep out of the Dervish fire lanes (§6.22: their
    // rifles reach 4, the forts' and guns' artillery 7; ours reach 5 and 8)
    // -- the side that outranges the enemy shoots without being shot. A Tomb
    // column pays the price (§9.14). (Historical: the army fights from the
    // Zariba hedge, §9.231, against a Dervish fire too weak to kite from.)
    if palace.is_none() && state.scenario != Scenario::Historical {
        let marching = tomb_objective(state, unit).is_some();
        let weight = if marching {
            0.05
        } else if is_leader {
            1.5
        } else if is_fragile {
            0.6
        } else {
            0.35
        };
        let lane = crate::threat::fire_reaching(state, at, player.opponent());
        value -= (LANE_FEAR * weight * lane) as i32;
        // A leader dies with his stack (§6.51): keep him beyond the reach
        // of the spears as well as the rifles.
        if is_leader && !marching {
            value -= (crate::threat::melee_reaching(state, at, player.opponent()) / 4.0) as i32;
        }
    }
    // A firing position (§6.2): the shot this unit would have from here in
    // the coming fire phase. The army wins by fire (§6.24's +1 is ours
    // alone); a gun line out of range or sight wins nothing. The spears
    // that could reach the hex are the price (§7.7: the Dervish melee at
    // +2).
    if !is_leader {
        value += (KITCHENER_SHOT_WEIGHT * crate::threat::best_shot_from(state, unit, at)) as i32;
        value -= (crate::threat::melee_reaching(state, at, player.opponent())
            * KITCHENER_SPEAR_FEAR) as i32;
    }
    // Company: a leader is never alone (§6.51); brigades like their
    // battalions together (§5.54).
    let friends = friends_in(state, at, player, unit.id);
    value += if is_leader && friends == 0 {
        -60
    } else if friends > 0 {
        6
    } else {
        0
    };
    value
}

/// Khalifa's value of `unit` standing on `at`: progress to its goal, massed
/// with its tribe, out of the lanes of the Maxims and gunboats unless in
/// contact (where the +2 melee lives, §7.7), guns out of melee reach.
fn khalifa_position_value(
    state: &GameState,
    unit: &omdurman_rules::UnitPlacement,
    at: HexCoord,
    goal: Option<Goal>,
    player: Player,
    palace: Option<HexCoord>,
    night: bool,
) -> i32 {
    let land = !unit.profile.kind.is_boat();
    let mut value = goal.map_or(0, |g| g.term(state, at, KHALIFA_PER_HEX, land));
    // Mass by tribe (§5.52 stacks are single-tribe anyway).
    value += friends_in(state, at, player, unit.id).min(3) * 4;
    // Fire (§6.7). In Fall of Khartoum the clock (§9.35) outranks blood: a
    // gentle penalty, halved at night. On the Omdurman maps the Maxims and
    // gunboats own the open ground (§6.22, §6.42): stand outside their lanes
    // and cross the killing ground only into contact.
    value -= if palace.is_some() {
        let exposure = adjacent_enemy_fire(state, at, player);
        if night { exposure / 15 } else { exposure / 8 }
    } else {
        lane_cost(state, at, player)
    };
    // Artillery is the breach key: keep it out of melee reach.
    if unit.profile.weapon == omdurman_rules::WeaponClass::Artillery
        && adjacent_enemy_stacks(state, at, player) > 0
    {
        value -= 25;
    }
    value
}

// ---------------------------------------------------------------------------
// Kitchener (Anglo-Egyptian)
// ---------------------------------------------------------------------------

fn kitchener_score(effect: &GameEffect, state: &GameState, player: Player) -> i32 {
    use GameEffect::*;
    let palace = palace_hex(state);
    match effect {
        // Placement matters: build full 4-stacks on the threatened wall line
        // (§5.51, §9.321), guns and leaders behind it.
        DeployUnit(p) => {
            if is_fixed_placement(p) {
                return 200;
            }
            let mut s = 60;
            if p.profile.kind.is_boat() {
                return s + 8; // Nile is fine; the zone bounds it (§5.22)
            }
            s += (state.units_in_hex(p.position).len() as i32).min(4) * 4;
            // Brigade integrity (§5.54): four battalions of one brigade in
            // one hex firing at one hex add +1 — mass the brigades.
            let same_brigade = state
                .units_in_hex(p.position)
                .into_iter()
                .filter(|u| {
                    p.profile.identity.brigade().is_some()
                        && u.profile.identity.brigade() == p.profile.identity.brigade()
                })
                .count() as i32;
            s += same_brigade.min(3) * 5;
            if on_city_line(state, p.position) {
                s += 8;
            }
            // Corridor geometry decides the whole defence (gates are
            // walkable, §5.23): the INSIDE of a gate is the load-bearing
            // plug; hexes staging next to an inside plug re-plug it; the
            // OUTSIDE of a gate is a trap — no wall at the back, the whole
            // horde melee-reaches it, and losing there opens the corridor.
            match corridor_side(state, p.position, palace) {
                Some(Side::Inside) => s += 28,
                Some(Side::Staging) => s += 14,
                Some(Side::Outside) => s -= 14,
                None => {}
            }
            if matches!(
                state.board.terrain_at(p.position),
                Some(
                    omdurman_types::Terrain::Building { .. } | omdurman_types::Terrain::Huts { .. }
                )
            ) {
                s += 3;
            }
            // Stand where the Dervish will come from (§9.322: south/east
            // edges): the nearer their mass (or entry edge, pre-deploy), the
            // better.
            let enemy_near: Option<i32> = {
                let e = player.opponent();
                state
                    .units
                    .iter()
                    .filter(|u| u.profile.identity.owner() == e)
                    .map(|u| u.position.distance(p.position) as i32)
                    .min()
            };
            s += match enemy_near {
                Some(d) => 12 - d.min(12),
                None => 12 - dist_to_dervish_entry(state, p.position).min(12),
            };
            s
        }
        PlaceReinforcements(_) => 70,
        ConfirmSetupReady { .. } => 12,
        // Counter-battery and soften the assault staging areas. Artillery
        // attacks at enemy artillery are the top priority: only artillery can
        // breach walls, sink gunboats, or destroy forts (§6.61-§6.63), and in
        // FoK every Dervish shell against a wall is a corridor to GORDON.
        FireCombat { attack, .. } | HowitzerFire { attack, .. } => {
            // Expected victory points destroyed (§6.22 CRT row, §6.23/§6.24/
            // §5.54 modifiers, §6.61/§6.62 special targets), with a premium
            // on the enemy guns that alone breach walls and sink gunboats.
            let targets = state.units_in_hex(attack.target_hex);
            let enemy = player.opponent();
            let has_artillery = targets.iter().any(|u| {
                u.profile.identity.owner() == enemy
                    && u.profile.weapon == omdurman_rules::WeaponClass::Artillery
                    && !matches!(u.profile.kind, omdurman_types::UnitKind::Fort { .. })
            });
            let near_palace = palace
                .map(|p| p.distance(attack.target_hex) <= 3)
                .unwrap_or(false);
            let value = crate::fire_plan::fire_value(state, attack)
                * if has_artillery { 1.6 } else { 1.0 }
                * if near_palace { 1.3 } else { 1.0 };
            if value < 0.02 {
                return 0;
            }
            30 + (value * 15.0).min(80.0) as i32
        }
        // Melee is the side's weakness (§7.7: +1 vs the Dervish +2): only
        // against a locally outnumbered, unsupported enemy.
        DeclareMelee { attack, .. } => {
            // Expected VP traded (§7.3/§7.7): the side's +1 against the
            // Dervish +2 makes most melees a loss -- take only the good ones.
            let value = crate::fire_plan::melee_value(state, attack);
            let support = adjacent_enemy_stacks(state, attack.defender_hex, player) - 1;
            if value > 0.4 && support <= 2 {
                40 + (value * 10.0).min(40.0) as i32
            } else {
                -40
            }
        }
        ResolveMelee => 50,
        // Cavalry/camel run rather than die (§7.5): retreat when outnumbered.
        RetreatBeforeMelee { unit_id, .. } => match state.pending_melee.as_ref() {
            Some(pending) => {
                let attackers = pending.attack.attackers.len() as i32;
                let defenders = pending.attack.defenders.len() as i32;
                if attackers >= defenders * 2 { 60 } else { -50 }
            }
            None => {
                let _ = unit_id;
                -50
            }
        },
        // Take vacated ground, but not into a massed counterattack (§6.82).
        AdvanceAfterCombat { unit_id, to } => {
            let exposure = adjacent_enemy_fire(state, *to, player);
            let heavy = exposure > 24 && !is_night(state);
            if heavy {
                8
            } else {
                30 + progress(state, *unit_id, *to, palace)
            }
        }
        MoveUnit { unit_id, to, .. } => {
            let Some(unit) = state.find_unit(*unit_id) else {
                return 0;
            };
            let goal = kitchener_goal(state, unit.id, player);
            let value = |at| kitchener_position_value(state, unit, at, goal, player, palace);
            move_score(value(*to) - value(unit.position))
        }
        Demolition { .. } => 45,
        DervishDesertion { .. } => 30,
        AdvancePhase => match state.phase {
            Phase::Setup => 12,
            _ => 1,
        },
        _ => 0,
    }
}

/// The Campaign's last turn (§9.12: 22 turns); the Tomb is scored at its end.
const CAMPAIGN_LAST_TURN: u8 = 22;

/// Movement points a Tomb column covers per remaining turn, with margin for
/// the night turns (§8.1 halves the Anglo-Egyptian movement) and blocking.
const TOMB_DASH_MP_PER_TURN: i32 = 4;

/// The Mahdi's Tomb, for the units that dash for it: on the Campaign map the
/// British leaders and the regular infantry (a leader plus an undisrupted
/// non-"Friendlies" combat unit on the hex at the end takes its 25 VP from
/// the Dervish, §9.14 -- a 50-VP swing). A unit sets off only when the turns
/// left just cover its path: a column that arrives early sits for turns
/// under the guns of the city's forts (§6.54) and dies there.
fn tomb_objective(state: &GameState, unit: &omdurman_rules::UnitPlacement) -> Option<HexCoord> {
    if state.scenario != Scenario::Campaign {
        return None;
    }
    let marches = match unit.profile.identity {
        UnitIdentity::AngloEgyptianLeader(_) => true,
        UnitIdentity::AngloEgyptianInfantry { .. } => !unit.profile.identity.is_friendlies(),
        _ => false,
    };
    if !marches {
        return None;
    }
    let tomb = state.board.hex_of_location(Location::MahdisTomb)?;
    let turns_left = i32::from(CAMPAIGN_LAST_TURN.saturating_sub(state.current_turn.value())) + 1;
    let due = |hex: HexCoord| {
        // Due: the turns left just cover the path. Still feasible: even an
        // unopposed march (~7 MP a turn) could make it -- otherwise call it
        // off rather than lose the column on the last turn.
        crate::threat::path_cost(state, hex, tomb).is_some_and(|cost| {
            turns_left * TOMB_DASH_MP_PER_TURN <= cost + TOMB_DASH_MP_PER_TURN
                && cost <= turns_left * 7
        })
    };
    if !due(unit.position) {
        return None;
    }
    // Without a leader who can still make it there is nothing to take; and
    // a city still full of the field army is a massacre, not a dash.
    let leader_due = state.units.iter().any(|u| {
        matches!(u.profile.identity, UnitIdentity::AngloEgyptianLeader(_))
            && crate::threat::path_cost(state, u.position, tomb)
                .is_some_and(|cost| cost <= turns_left * TOMB_DASH_MP_PER_TURN + 4)
    });
    // The forts ring the city (§9.111) and fire every turn even alone
    // (§6.54): measured in arena play, a dash past them cost the leaders
    // and a score of battalions -- more than the Tomb's 50-VP swing. Go
    // only once the guns have cleared the city.
    let defenders = state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == Player::Dervish && u.position.distance(tomb) <= 7)
        .count();
    (leader_due && defenders <= 3).then_some(tomb)
}

/// Kitchener's goal for one unit: hold the threatened gate/breach corridors
/// and the palace ring in Fall of Khartoum; on the campaign map, close on
/// the field army at rifle range (forts last), leaders sheltering with the
/// safest stack, the Tomb column dashing when it is due.
fn kitchener_goal(state: &GameState, unit_id: UnitId, player: Player) -> Option<Goal> {
    let unit = state.find_unit(unit_id)?;
    let enemy = player.opponent();
    if let Some(tomb) = tomb_objective(state, unit) {
        return Some(Goal::March(tomb));
    }
    match unit.profile.identity {
        // Leaders bodyguard toward the nearest friendly stack (§6.51).
        UnitIdentity::AngloEgyptianLeader(_) => {
            if palace_hex(state).is_some() {
                nearest_friendly_stack(state, unit.position, unit_id).map(Goal::Hex)
            } else {
                // Off the FoK walls a leader is 10 VP (§9.14) and dies with
                // his stack (§6.51): shelter with the safest stack near by.
                safest_friendly_stack(state, unit.position, player).map(Goal::Hex)
            }
        }
        // Guns stand off at their own range (§6.22): the batteries'
        // artillery at full strength out to six hexes, beyond the Dervish
        // rifles' four; the Maxims at their normal range of three.
        UnitIdentity::AngloEgyptianArtillery => {
            let nearest_enemy = nearest_enemy_unit(state, unit.position, enemy);
            nearest_enemy.map(|target| Goal::Band {
                target,
                standoff: 6,
            })
        }
        UnitIdentity::AngloEgyptianMaxim => {
            let nearest_enemy = nearest_enemy_unit(state, unit.position, enemy);
            nearest_enemy.map(|target| Goal::Band {
                target,
                standoff: 5,
            })
        }
        // Gunboats hunt along the river at Maxim range (§6.22, §6.42); only
        // artillery can hurt them (§6.61), and its reach is in the fire
        // lanes of the position value.
        UnitIdentity::AngloEgyptianGunboat(_) => {
            // The quarry nearest by water, not by line: the enemy the boat
            // can bring under its guns soonest (see `threat::river_cost`).
            let mut stacks: Vec<HexCoord> = state
                .units
                .iter()
                .filter(|u| u.profile.identity.owner() == enemy)
                .map(|u| u.position)
                .collect();
            stacks.sort_by_key(|p| (p.q, p.r));
            stacks.dedup();
            stacks
                .into_iter()
                // A target near the bank, within the Maxims' reach from the
                // river, outranks a nearer one inland the boat can only
                // shell from afar.
                .min_by_key(|&p| {
                    crate::threat::river_field(state, unit.position, p, 16).unwrap_or(i32::MAX)
                })
                .map(|target| Goal::Band {
                    target,
                    standoff: 2,
                })
        }
        // Everything else holds the line.
        _ => {
            if let Some(palace) = palace_hex(state) {
                // FoK defence in depth, in priority order:
                // 1. An undermanned gate/breach plug (gates are walkable,
                //    §5.23 — an open corridor is GORDON's death warrant);
                // 2. The interior of the corridor nearest the DERVISH threat;
                // 3. The palace ring as the last-line reserve (§9.346).
                let threat = nearest_enemy_unit(state, palace, enemy)
                    .or_else(|| nearest_enemy_unit(state, unit.position, enemy));
                let plug = gate_plug_vacancy(state, player, palace);
                let corridor = threat.and_then(|t| nearest_corridor_inside(state, t, palace));
                if let Some(p) = plug {
                    return Some(Goal::Hex(p));
                }
                match corridor {
                    Some(c) if combat_count_in(state, c, player) < 4 => Some(Goal::Hex(c)),
                    // The palace ring: the last-line reserve (§9.346).
                    _ => Some(Goal::Band {
                        target: palace,
                        standoff: 1,
                    }),
                }
            } else {
                // Omdurman maps: a formed line at rifle range of the field
                // army. The forts and the walled city's garrison are not
                // the field army: forts never come out (§5.25), only guns
                // can hurt them (§6.62) and they are worth nothing (§9.14);
                // the garrison sits behind walls (§5.23, §7.2). With the
                // field army gone the line has no goal left -- it keeps out
                // of the guns' reach (the fire lanes of the position value)
                // unless the Tomb column marches (`tomb_objective`).
                let field = state
                    .units
                    .iter()
                    .filter(|u| {
                        u.profile.identity.owner() == enemy
                            && u.profile.identity != UnitIdentity::DervishFort
                            && !state.board.walled_city.contains(&u.position)
                    })
                    .map(|u| u.position)
                    .min_by_key(|p| p.distance(unit.position));
                field.map(|target| Goal::Band {
                    target,
                    // Behind the Zariba (§9.231) against a Dervish fire too
                    // weak to fear, the rifles' normal range (§6.22); in the
                    // open, one hex beyond the Dervish rifles' reach (four).
                    standoff: if state.scenario == Scenario::Historical {
                        3
                    } else {
                        5
                    },
                })
            }
        }
    }
}

/// The nearest gate/breach inside hex not yet held 4-strong: the continuous
/// re-plug duty (§5.23, §9.346). Gates are walkable, so a corridor held by
/// fewer than four defenders is the assault's way in.
fn gate_plug_vacancy(state: &GameState, player: Player, palace: HexCoord) -> Option<HexCoord> {
    let mut plugs: Vec<HexCoord> = Vec::new();
    for (hr, _) in &state.board.hexsides {
        if !matches!(
            state.hexside_effective(hr.a, hr.b),
            Some(HexsideKind::Gate | HexsideKind::Breach)
        ) {
            continue;
        }
        let inside = if palace.distance(hr.a) <= palace.distance(hr.b) {
            hr.a
        } else {
            hr.b
        };
        if !plugs.contains(&inside) {
            plugs.push(inside);
        }
    }
    plugs.into_iter().min_by_key(|p| {
        (
            combat_count_in(state, *p, player),
            p.distance(palace) as i32,
        )
    })
}

/// The gate/breach corridor on the *defended* side (closer to the palace than
/// the outside) nearest to `from`, so garrison units plug the walls (§5.23).
fn nearest_corridor_inside(
    state: &GameState,
    from: HexCoord,
    palace: HexCoord,
) -> Option<HexCoord> {
    let mut best: Option<(i32, HexCoord)> = None;
    for (hr, _) in &state.board.hexsides {
        if !matches!(
            state.hexside_effective(hr.a, hr.b),
            Some(HexsideKind::Gate | HexsideKind::Breach)
        ) {
            continue;
        }
        // The inside endpoint is the one closer to the palace.
        let inside = if palace.distance(hr.a) <= palace.distance(hr.b) {
            hr.a
        } else {
            hr.b
        };
        let d = from.distance(inside) as i32;
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, inside));
        }
    }
    best.map(|(_, hex)| hex)
}

// ---------------------------------------------------------------------------
// Khalifa (Dervish)
// ---------------------------------------------------------------------------

fn khalifa_score(effect: &GameEffect, state: &GameState, player: Player) -> i32 {
    use GameEffect::*;
    let palace = palace_hex(state);
    let night = is_night(state);
    let axis = assault_axis_wall(state);
    match effect {
        // Concentrate the wave on the assault axis: same-tribe stacks near
        // the chosen wall segment, guns close enough to breach on turn 1
        // (§6.63, §9.322).
        DeployUnit(p) => {
            if is_fixed_placement(p) {
                return 200;
            }
            let mut s = 60;
            if p.profile.kind.is_boat() {
                return s;
            }
            let same_tribe = state
                .units_in_hex(p.position)
                .into_iter()
                .filter(|u| {
                    u.profile.identity.owner() == player
                        && u.profile.identity.faction() == p.profile.identity.faction()
                })
                .count() as i32;
            s += same_tribe.min(3) * 5;
            let anchor = axis.or(palace);
            if let Some(a) = anchor {
                let d = p.position.distance(a) as i32;
                let weight = if p.profile.weapon == omdurman_rules::WeaponClass::Artillery {
                    4
                } else {
                    3
                };
                s += 24 - weight * d.min(24);
            }
            s
        }
        PlaceReinforcements(_) => 75,
        ConfirmSetupReady { .. } => 12,
        // Breach the wall (§6.63): the door to GORDON. Prefer walls close to
        // the assault mass and on the palace axis.
        ArtilleryBreachWall { firers, target, .. } => {
            // The breach sits between hexes a and b; score both endpoints and
            // keep the better (nearer the assault axis).
            let end = |hex: omdurman_types::HexCoord| -> i32 {
                let mass_dist = firers
                    .first()
                    .and_then(|id| state.find_unit(*id))
                    .map(|u| u.position)
                    .map_or(99, |_| {
                        state
                            .units
                            .iter()
                            .filter(|u| u.profile.identity.owner() == player)
                            .map(|u| u.position.distance(hex))
                            .min()
                            .unwrap_or(99)
                    });
                let axis = palace
                    .map(|p| 10 - (p.distance(hex) as i32).min(10))
                    .unwrap_or(0);
                60 + axis - mass_dist as i32
            };
            end(target.a).max(end(target.b))
        }
        // Soften the garrison stacks the melee wave is about to hit; the
        // garrison holding the breach corridor is the priority target.
        FireCombat { attack, .. } | HowitzerFire { attack, .. } => {
            // Expected victory points destroyed (see `fire_plan`), weighted
            // toward the garrison of the assault axis.
            let on_axis = axis
                .map(|a| a.distance(attack.target_hex) <= 2)
                .unwrap_or(false);
            let value =
                crate::fire_plan::fire_value(state, attack) * if on_axis { 1.4 } else { 1.0 };
            if value < 0.02 {
                return 0;
            }
            25 + (value * 12.0).min(80.0) as i32
        }
        // Melee is where the +2 lives (§7.7): attack with local mass, at the
        // hex that leads to the palace, under night cover when possible.
        DeclareMelee { attack, .. } => {
            let attackers = attack.attackers.len() as i32;
            let defenders = combat_count_in(state, attack.defender_hex, player.opponent());
            // Expected VP traded (§7.3/§7.7, AE units are 3 VP to the
            // Dervish 1): melee where the +2 pays, not into a losing trade.
            let value = crate::fire_plan::melee_value(state, attack);
            let racing = palace.is_some();
            if value < if racing { -1.0 } else { 0.2 } {
                return -40;
            }
            let softened = state
                .units_in_hex(attack.defender_hex)
                .iter()
                .any(|u| u.profile.identity.owner() == player.opponent() && u.state.disrupted);
            let forward = palace
                .map(|p| {
                    let here = attack.attacker_hex.distance(p) as i32;
                    let there = attack.defender_hex.distance(p) as i32;
                    (here - there) * 5
                })
                .unwrap_or(0);
            let support = adjacent_enemy_stacks(state, attack.defender_hex, player) - 1;
            let on_axis = assault_axis_wall(state)
                .map(|a| a.distance(attack.defender_hex) <= 2)
                .unwrap_or(false);
            40 + (value * 6.0).clamp(-6.0, 30.0) as i32 + (attackers - defenders) * 2 + forward
                - support * 6
                + i32::from(night) * 6
                + i32::from(on_axis) * 6
                + i32::from(softened) * 10
        }
        ResolveMelee => 55,
        // The Khalifa's body is 10 VP (§9.14): the leader runs from melee.
        RetreatBeforeMelee { unit_id, .. } => {
            let is_leader = state
                .find_unit(*unit_id)
                .is_some_and(|u| matches!(u.profile.identity, UnitIdentity::DervishLeader(_)));
            if is_leader { 70 } else { -30 }
        }
        // The mandatory pour-through is the breakthrough (§7.6): always take
        // it toward the objective.
        AdvanceAfterCombat { unit_id, to } => 50 + 6 * progress(state, *unit_id, *to, palace),
        MoveUnit { unit_id, to, .. } => {
            let Some(unit) = state.find_unit(*unit_id) else {
                return 0;
            };
            let goal = khalifa_goal(state, unit.id, player);
            let value = |at| khalifa_position_value(state, unit, at, goal, player, palace, night);
            move_score(value(*to) - value(unit.position))
        }
        Demolition { .. } => 45,
        DervishDesertion { .. } => 30,
        AdvancePhase => match state.phase {
            Phase::Setup => 12,
            _ => 1,
        },
        _ => 0,
    }
}

/// The Dervish cost of standing on `at` under Anglo-Egyptian fire (see
/// [`crate::threat`]): a bonus in contact (the melee is the point),
/// otherwise the fire that reaches the hex, lighter at night (§8.1 halves
/// every range).
fn lane_cost(state: &GameState, at: HexCoord, player: Player) -> i32 {
    let enemy = player.opponent();
    let contact = at.neighbors().iter().any(|&n| {
        state
            .units_in_hex(n)
            .iter()
            .any(|u| u.profile.identity.owner() == enemy)
            && !matches!(
                state.hexside_effective(at, n),
                Some(HexsideKind::Wall | HexsideKind::ZaribaThornHedge | HexsideKind::ZaribaTrench)
            )
    });
    if contact {
        return -6;
    }
    // Historical: four turns against a Zariba that cannot be meleed across
    // (§9.231) -- every unit kept out of the fire is a step down the
    // Anglo-Egyptian §9.24 schedule.
    let weight = match (state.scenario, is_night(state)) {
        (Scenario::Historical, false) => 0.8,
        (Scenario::Historical, true) => 0.3,
        (_, false) => 0.25,
        (_, true) => 0.08,
    };
    (weight * crate::threat::fire_reaching(state, at, enemy)) as i32
}

/// Khalifa's goal for one unit: the assault corridor (gate or breach) until
/// the way in is open, then the palace (§9.346); artillery stays in breach
/// range of the axis wall; the bodyguard shadows the Khalifa on the campaign
/// map.
fn khalifa_goal(state: &GameState, unit_id: UnitId, player: Player) -> Option<Goal> {
    let unit = state.find_unit(unit_id)?;
    let enemy = player.opponent();
    if let Some(palace) = palace_hex(state) {
        let axis = assault_axis_wall(state);
        let is_guns = unit.profile.weapon == omdurman_rules::WeaponClass::Artillery;
        let corridor = assault_corridor(state);
        // Already through the corridor line (or in across any open gate):
        // everything past this point is a footrace to GORDON (§9.346). The
        // gate is walkable (§5.23), so no melee is needed until a defender
        // appears — the goal just has to stop flapping back outside.
        let inside = corridor
            .map(|c| unit.position.distance(palace) < c.distance(palace))
            .unwrap_or(false)
            || unit.position.neighbors().iter().any(|&n| {
                matches!(
                    state.hexside_effective(unit.position, n),
                    Some(HexsideKind::Breach) | Some(HexsideKind::Gate)
                ) && n.distance(palace) < unit.position.distance(palace)
            });
        if inside {
            return Some(Goal::Hex(palace));
        }
        if is_guns && !crate::aggressive::any_breach_exists(state) {
            // Guns before the first breach: the axis wall, in breach range
            // (§6.63).
            return axis.map(Goal::Hex);
        }
        // Everyone else: the corridor if one is open nearby, else mass on
        // the axis wall outside and wait for the guns.
        if let Some(c) = corridor {
            return Some(Goal::Hex(c));
        }
        return axis.map(Goal::Hex);
    }
    // The Mahdi's Tomb is 25 VP to whoever holds it at the end (§9.14), and
    // the Anglo-Egyptians take it only by standing on it: the Khalifa and
    // his Taiasha bodyguard (the only tribe allowed in the walled city,
    // §5.23) garrison it.
    if state.scenario == Scenario::Campaign
        && matches!(
            unit.profile.identity,
            UnitIdentity::DervishLeader(omdurman_rules::DervishLeader::KhalifaAbdullah)
                | UnitIdentity::DervishTribal {
                    tribe: omdurman_types::DervishTribe::Taiasha
                }
        )
        && let Some(tomb) = state.board.hex_of_location(Location::MahdisTomb)
    {
        return Some(Goal::March(tomb));
    }
    match unit.profile.identity {
        // The Khalifa stays guarded (10 VP, §9.14): hover behind the line.
        UnitIdentity::DervishLeader(_) => {
            let nearest_enemy = nearest_enemy_unit(state, unit.position, enemy);
            nearest_enemy.map(|target| Goal::Band {
                target,
                standoff: 4,
            })
        }
        UnitIdentity::DervishArtillery => {
            let nearest_enemy = nearest_enemy_unit(state, unit.position, enemy);
            nearest_enemy.map(|target| Goal::Band {
                target,
                standoff: 3,
            })
        }
        _ => nearest_enemy_unit(state, unit.position, enemy).map(Goal::Hex),
    }
}

// ---------------------------------------------------------------------------
// Geometry helpers shared by both commanders
// ---------------------------------------------------------------------------

/// Position of the enemy unit closest to `from`.
fn nearest_enemy_unit(state: &GameState, from: HexCoord, enemy: Player) -> Option<HexCoord> {
    state
        .units
        .iter()
        .filter(|u| u.profile.identity.owner() == enemy)
        .map(|u| u.position)
        .min_by_key(|p| p.distance(from))
}

/// The nearest hex holding at least one friendly combat unit other than
/// `unit_id` itself (the bodyguard destination, §6.51).
fn nearest_friendly_stack(state: &GameState, from: HexCoord, unit_id: UnitId) -> Option<HexCoord> {
    state
        .units
        .iter()
        .filter(|u| {
            u.id != unit_id
                && u.profile.identity.owner()
                    == state
                        .find_unit(unit_id)
                        .map(|x| x.profile.identity.owner())
                        .unwrap_or(Player::AngloEgyptian)
                && !matches!(
                    u.profile.identity,
                    UnitIdentity::AngloEgyptianLeader(_) | UnitIdentity::DervishLeader(_)
                )
        })
        .map(|u| u.position)
        .min_by_key(|p| p.distance(from))
}

/// The friendly combat stack a leader should shelter with: within 8 hexes,
/// the one least exposed to enemy fire (see [`crate::threat`]) and not in
/// contact, nearer ones preferred.
fn safest_friendly_stack(state: &GameState, from: HexCoord, player: Player) -> Option<HexCoord> {
    let enemy = player.opponent();
    let mut best: Option<(i32, HexCoord)> = None;
    for u in &state.units {
        if u.profile.identity.owner() != player
            || matches!(
                u.profile.identity,
                UnitIdentity::AngloEgyptianLeader(_)
                    | UnitIdentity::AngloEgyptianGunboat(_)
                    | UnitIdentity::AngloEgyptianFort
            )
        {
            continue;
        }
        let d = u.position.distance(from) as i32;
        if d > 8 {
            continue;
        }
        let contact = u.position.neighbors().iter().any(|&n| {
            state
                .units_in_hex(n)
                .iter()
                .any(|e| e.profile.identity.owner() == enemy)
        });
        let threat = crate::threat::fire_reaching(state, u.position, enemy) as i32;
        let spears = crate::threat::melee_reaching(state, u.position, enemy) as i32;
        let key = threat * 2 + spears + i32::from(contact) * 100 + d;
        if best.is_none_or(|(k, _)| key < k) {
            best = Some((key, u.position));
        }
    }
    best.map(|(_, h)| h).or_else(|| {
        state
            .units
            .iter()
            .filter(|u| {
                u.profile.identity.owner() == player
                    && !matches!(u.profile.identity, UnitIdentity::AngloEgyptianLeader(_))
            })
            .map(|u| u.position)
            .min_by_key(|p| p.distance(from))
    })
}

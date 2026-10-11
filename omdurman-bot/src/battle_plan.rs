//! The battles of 1 and 2 September 1898 as the two commanders fought
//! them, for the Historical scenario (§9.2) and the Campaign game (§9.1).
//!
//! **The Historical scenario** -- 2 September, 6:00 to noon, four turns,
//! the Dervish moving first (§9.212). Each commander's goals follow what
//! the two armies did that morning, read onto the scenario's set-up (the
//! lettered hexes of §9.212, the Zariba and the Kerreri detachment of
//! §9.211):
//!
//! - **Dawn (turn 1).** The white flags and Sherif's red banner on the
//!   Dervish right (Osman Azrak's assault; here Osman Digna's and Sherif's
//!   commands, letters O and S) go straight at the Zariba. The Green Flag
//!   (Ali Wad Helu and Sheik El Din, letters A and D) swings north over the
//!   Kerreri hills after the mounted troops at Kerreri. The Black Flag (the
//!   Khalifa and Yakub, letters K and Y) waits behind Jebel Surgham. The
//!   army holds the Zariba and fires; the Kerreri detachment -- Camel
//!   Corps, Egyptian cavalry, horse battery -- falls back on it along the
//!   river.
//! - **The advance (turns 2-4).** The army leaves the Zariba and marches on
//!   Omdurman in echelon from the left: the 2nd British Brigade by the Nile
//!   leading, then the 1st British, the 2nd and 3rd Egyptian, Macdonald's
//!   1st Egyptian at the rear right, the 4th Egyptian as rearguard, the
//!   21st Lancers at the head. A brigade the enemy closes on halts and
//!   fights. The Black Flag falls on whatever has left the Zariba and comes
//!   within a rush; the Green Flag comes back off the hills to do the same;
//!   Osman Digna's survivors lie in the khor and spear what comes within
//!   reach. A command out of reach stays out of the guns' sight.
//!
//! **The Campaign** -- 1 September (turns 1-8), the night (9-10), 2
//! September (11-18, §9.12). The army comes down the Nile to El Egeiga,
//! builds the Zariba (§9.23) and waits in it through the night and the
//! dawn attack; at half past eight it marches on Omdurman as above, and
//! the Tomb column goes in at the end (see `commanders::tomb_objective`).
//! The Dervish field army marches out and gathers on the plain west of
//! Jebel Surgham, falls on what comes within a rush, and attacks at dawn.
//! The gunboats shell the city's forts and the Tomb meanwhile (the
//! commanders' own hunting).
//!
//! The goals feed the commanders' position values (see
//! [`crate::commanders`]); everything else -- fire, melee, the fear of the
//! Maxims -- stays the commanders' own.

use omdurman_rules::effects::{GameState, HISTORICAL_KERRERI_UNITS};
use omdurman_rules::{DervishLeader, UnitIdentity, UnitPlacement};
use omdurman_types::{BrigadeId, DervishTribe, HexCoord, Player, Scenario};

use crate::commanders::Goal;

/// The Dervish commands of the morning, by the colour of their leader
/// (§9.212).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Flag {
    /// Ali Wad Helu and Sheik El Din (A, D): north-west, behind Kerreri.
    Green,
    /// The Khalifa and Yakub (K, Y): behind Jebel Surgham.
    Black,
    /// Sherif (S): the red banner on the right, in the dawn assault, then
    /// with the Black Flag.
    Red,
    /// Osman Digna (O): the white flags on the right, in the dawn assault,
    /// then in the khor.
    White,
}

fn flag_of_leader(leader: DervishLeader) -> Flag {
    match leader {
        DervishLeader::AliWadHelu | DervishLeader::SheikElDin => Flag::Green,
        DervishLeader::KhalifaAbdullah | DervishLeader::Yakub => Flag::Black,
        DervishLeader::Sherif => Flag::Red,
        DervishLeader::OsmanDigna => Flag::White,
    }
}

/// The leader whose colour a Dervish unit wears (§9.212); the guns are the
/// Khalifa's.
fn leader_of(identity: &UnitIdentity) -> Option<DervishLeader> {
    match identity {
        UnitIdentity::DervishLeader(l) => Some(*l),
        UnitIdentity::DervishTribal { tribe } => DervishLeader::of_tribe(*tribe),
        UnitIdentity::DervishArtillery => Some(DervishLeader::KhalifaAbdullah),
        _ => None,
    }
}

/// How far (hexes) the ambush in the khor reaches out: the spears' rush,
/// not a march.
const KHOR_REACH: u32 = 5;
/// An Anglo-Egyptian brigade with the enemy this close halts and fights.
const ENGAGE_RANGE: u32 = 5;

/// A land unit's printed movement allowance in hexes of clear ground
/// (§5.11): how far it can strike in one move.
fn movement_allowance(unit: &UnitPlacement) -> u32 {
    match unit.profile.movement {
        omdurman_rules::UnitMovement::Land(mp) => u32::from(mp.value()),
        _ => 0,
    }
}

/// Anglo-Egyptian land units in the open: off the Zariba (§9.23), not
/// leaders (they have no fight in them, §6.51).
fn exposed_enemy(state: &GameState) -> impl Iterator<Item = &UnitPlacement> {
    state.units.iter().filter(|u| {
        u.profile.identity.owner() == Player::AngloEgyptian
            && !u.profile.kind.is_boat()
            && !matches!(u.profile.identity, UnitIdentity::AngloEgyptianLeader(_))
            && !state.board.is_zariba(u.position)
    })
}

/// The nearest unit of `units` that a land unit at `from` can reach the
/// side of in `reach` movement points (§5.11: the ground counts, a hex of
/// the Surgham ridge is not a hex of open plain); `u32::MAX` for any.
fn nearest_within<'a>(
    state: &GameState,
    units: impl Iterator<Item = &'a UnitPlacement>,
    from: HexCoord,
    reach: u32,
) -> Option<HexCoord> {
    let mut targets: Vec<HexCoord> = units
        .map(|u| u.position)
        .filter(|p| p.distance(from) <= reach)
        .collect();
    targets.sort_by_key(|p| (p.distance(from), p.q, p.r));
    targets.dedup();
    if reach == u32::MAX {
        return targets.first().copied();
    }
    // The target's own hex costs a step more than its side.
    targets.into_iter().find(|&p| {
        crate::threat::path_cost(state, from, p).is_some_and(|cost| cost <= reach as i32 + 1)
    })
}

/// Where a leader stands: with the largest stack of his colour (§5.53 --
/// he may stack only with it), the one nearest him on a tie. A leader is a
/// combat unit like any other (§6.5), and a lone one is a free kill.
fn with_his_command(state: &GameState, leader: DervishLeader, from: HexCoord) -> Option<HexCoord> {
    let mut counts: Vec<(HexCoord, usize)> = Vec::new();
    for u in &state.units {
        let UnitIdentity::DervishTribal { tribe } = u.profile.identity else {
            continue;
        };
        if DervishLeader::of_tribe(tribe) != Some(leader) {
            continue;
        }
        match counts.iter_mut().find(|(h, _)| *h == u.position) {
            Some((_, n)) => *n += 1,
            None => counts.push((u.position, 1)),
        }
    }
    counts
        .into_iter()
        .max_by_key(|&(h, n)| (n, std::cmp::Reverse(h.distance(from)), h.q, h.r))
        .map(|(h, _)| h)
}

/// The Khalifa's goal for a Dervish unit (see the module notes); `None`
/// leaves the commander's own.
pub(crate) fn khalifa_goal(state: &GameState, unit: &UnitPlacement) -> Option<Goal> {
    match state.scenario {
        Scenario::Historical => khalifa_historical_goal(state, unit),
        Scenario::Campaign => khalifa_campaign_goal(state, unit),
        Scenario::FallOfKhartoum => None,
    }
}

/// Where the field army gathers on 1 September: the plain west of Jebel
/// Surgham, out of sight of the river (printed column 24, row 5).
const CAMPAIGN_ASSEMBLY: (i32, i32) = (24, 5);

/// The Campaign's Dervish field army: on 1 September it marches out of
/// Omdurman and falls on whatever of the enemy it finds on the plain; while
/// the enemy sits behind his Zariba it gathers west of Jebel Surgham, out of
/// reach of the guns, and attacks at dawn on the 2nd
/// (`CAMPAIGN_DAWN_TURN`) it attacks. The Khalifa and his Taiasha keep
/// the Tomb, the guns their own counsel (the commander's goals).
fn khalifa_campaign_goal(state: &GameState, unit: &UnitPlacement) -> Option<Goal> {
    if state.current_turn.value() >= CAMPAIGN_DAWN_TURN {
        return None;
    }
    let tribe = match unit.profile.identity {
        UnitIdentity::DervishTribal { tribe } => tribe,
        UnitIdentity::DervishLeader(l) if l != DervishLeader::KhalifaAbdullah => {
            return with_his_command(state, l, unit.position).map(Goal::Hex);
        }
        _ => return None,
    };
    if matches!(tribe, DervishTribe::Taiasha | DervishTribe::IsaZachneih) {
        return None;
    }
    // An army caught on the plain is attacked wherever it is; the Zariba
    // is not, before dawn.
    let target = nearest_within(state, exposed_enemy(state), unit.position, u32::MAX);
    Some(match target {
        Some(target) => Goal::Hex(target),
        None => Goal::Hex(on_land(
            state,
            printed(CAMPAIGN_ASSEMBLY.0, CAMPAIGN_ASSEMBLY.1),
        )),
    })
}

fn khalifa_historical_goal(state: &GameState, unit: &UnitPlacement) -> Option<Goal> {
    let leader = leader_of(&unit.profile.identity)?;
    if let UnitIdentity::DervishLeader(l) = unit.profile.identity {
        return with_his_command(state, l, unit.position).map(Goal::Hex);
    }
    let turn = state.current_turn.value();
    let hold = Goal::Hex(unit.position);
    let guns = unit.profile.identity == UnitIdentity::DervishArtillery;
    // A command falls on what it can reach in one move -- across the open
    // ground in daylight in a single rush, not halfway into the Maxims'
    // reach to wait there a turn (§5.11, §6.22).
    let one_move = movement_allowance(unit);
    let strike = |reach: u32| {
        nearest_within(state, exposed_enemy(state), unit.position, reach).map(|target| {
            if guns {
                Goal::Band {
                    target,
                    standoff: 3,
                }
            } else {
                Goal::Hex(target)
            }
        })
    };
    let goal = match (flag_of_leader(leader), turn) {
        // The dawn assault: straight at the hedge, the way a man walks --
        // not round by the river ends the Zariba is entered through
        // (§9.233).
        (Flag::Red | Flag::White, 1) => state
            .units
            .iter()
            .filter(|u| {
                u.profile.identity.owner() == Player::AngloEgyptian
                    && state.board.is_zariba(u.position)
            })
            .map(|u| u.position)
            .min_by_key(|p| (p.distance(unit.position), p.q, p.r))
            .map(Goal::Close)
            .or_else(|| strike(one_move))
            .unwrap_or(hold),
        (Flag::White, _) => strike(KHOR_REACH).unwrap_or(hold),
        (Flag::Black, 1) => hold,
        (Flag::Black | Flag::Red, _) => strike(one_move).unwrap_or(hold),
        // Turn 1: over the Kerreri hills after the mounted troops there.
        (Flag::Green, 1) => nearest_within(
            state,
            exposed_enemy(state).filter(|u| HISTORICAL_KERRERI_UNITS.contains(&u.id)),
            unit.position,
            u32::MAX,
        )
        .map(Goal::Hex)
        .or_else(|| strike(one_move))
        .unwrap_or(hold),
        (Flag::Green, _) => strike(one_move).unwrap_or(hold),
    };
    Some(goal)
}

/// A hex given in the board's printed columns and rows (`q`, and the row
/// `r - q / 2` down the page) as an axial coordinate.
fn printed(q: i32, row: i32) -> HexCoord {
    HexCoord::new(q, row + q.div_euclid(2))
}

/// The head of the column on `turn`: by the Nile past Jebel Surgham, then on
/// to the city -- a day's march of eight-hex turns (§5.11), the river on the
/// left.
fn march_point(turn: u8) -> (i32, i32) {
    match turn {
        0..=2 => (35, 4),
        3 => (34, 9),
        _ => (33, 14),
    }
}

/// A brigade's place in the echelon, in printed columns west and rows back
/// from the head of the column.
fn echelon_slot(brigade: BrigadeId) -> (i32, i32) {
    match (brigade.designation().nationality, brigade.number) {
        // Lyttelton, by the river, leading.
        (omdurman_types::BrigadeNationality::British, 2) => (0, 0),
        // Wauchope.
        (omdurman_types::BrigadeNationality::British, _) => (-2, -1),
        // Maxwell, Lewis.
        (_, 2) => (-4, -2),
        (_, 3) => (-6, -3),
        // Macdonald, at the rear right -- within rifle range of Lewis
        // (§6.22), so the brigades' fire covers each other's front.
        (_, 1) => (-7, -4),
        // Collinson, the rearguard.
        _ => (-3, -5),
    }
}

/// The nearest land hex of the board to `want` (the slots may fall on the
/// river).
fn on_land(state: &GameState, want: HexCoord) -> HexCoord {
    if state.board.terrain.contains_key(&want) && !state.board.is_nile(want) {
        return want;
    }
    state
        .board
        .terrain
        .keys()
        .copied()
        .filter(|&h| !state.board.is_nile(h))
        .min_by_key(|h| (h.distance(want), h.q, h.r))
        .unwrap_or(want)
}

/// The slot of `brigade` on `turn`.
fn slot(state: &GameState, brigade: BrigadeId, turn: u8) -> HexCoord {
    let (q, row) = march_point(turn);
    let (dq, drow) = echelon_slot(brigade);
    on_land(state, printed(q + dq, row + drow))
}

/// The nearest Zariba hex to `from`.
fn nearest_zariba(state: &GameState, from: HexCoord) -> Option<HexCoord> {
    state
        .board
        .zariba
        .iter()
        .copied()
        .min_by_key(|h| (h.distance(from), h.q, h.r))
}

/// The Campaign turn the Dervish attack: the dawn of 2 September (§9.12:
/// turns 9 and 10 are the night between).
const CAMPAIGN_DAWN_TURN: u8 = 11;

/// The Campaign turn the army leaves its Zariba for Omdurman: once the dawn
/// attack has broken on it (half past eight on 2 September).
pub(crate) const CAMPAIGN_ADVANCE_TURN: u8 = 12;

/// Kitchener's goal for an Anglo-Egyptian unit (see the module notes):
/// `Some(goal)` overrides the commander's own, `None` leaves it (the
/// leaders, the gunboats, the "Friendlies" across the river, and every unit
/// once the plan has played out).
///
/// In the Campaign the army comes down the Nile on 1 September to the
/// Zariba at El Egeiga and builds it (§9.23), sits out the night and the
/// dawn attack in it, and then marches on Omdurman like the Historical army.
pub(crate) fn kitchener_goal(state: &GameState, unit: &UnitPlacement) -> Option<Goal> {
    match state.scenario {
        Scenario::Historical => kitchener_historical_goal(state, unit),
        Scenario::Campaign => kitchener_campaign_goal(state, unit),
        Scenario::FallOfKhartoum => None,
    }
}

fn kitchener_campaign_goal(state: &GameState, unit: &UnitPlacement) -> Option<Goal> {
    let identity = unit.profile.identity;
    if unit.profile.kind.is_boat()
        || identity.is_friendlies()
        || matches!(identity, UnitIdentity::AngloEgyptianLeader(_))
    {
        return None;
    }
    let turn = state.current_turn.value();
    if turn < CAMPAIGN_ADVANCE_TURN {
        if state.board.is_zariba(unit.position) {
            return Some(Goal::Hex(unit.position));
        }
        // On the way down the river the column keeps marching: it fights
        // from the hedge, not out on the plain.
        return nearest_zariba_with_room(state, unit).map(Goal::March);
    }
    // The advance: the Historical march, two days on.
    let march_turn = turn - CAMPAIGN_ADVANCE_TURN + 2;
    if march_turn > 4 {
        return None;
    }
    advance_goal(state, unit, march_turn)
}

/// The nearest Zariba hex `unit` may still stack into (§5.51).
fn nearest_zariba_with_room(state: &GameState, unit: &UnitPlacement) -> Option<HexCoord> {
    state
        .board
        .zariba
        .iter()
        .copied()
        .filter(|&h| state.check_stacking(unit, h).is_ok())
        .min_by_key(|h| (h.distance(unit.position), h.q, h.r))
}

fn kitchener_historical_goal(state: &GameState, unit: &UnitPlacement) -> Option<Goal> {
    let identity = unit.profile.identity;
    if unit.profile.kind.is_boat() || matches!(identity, UnitIdentity::AngloEgyptianLeader(_)) {
        return None;
    }
    let turn = state.current_turn.value();
    if turn <= 1 {
        // Dawn: the army stands in the Zariba (§9.23); the Kerreri
        // detachment falls back on it along the river, under the gunboats --
        // mounted troops caught in the open by the Green Flag are lost.
        if state.board.is_zariba(unit.position) {
            return Some(Goal::Hex(unit.position));
        }
        return nearest_zariba(state, unit.position).map(Goal::March);
    }
    advance_goal(state, unit, turn)
}

/// A unit's goal on the march on Omdurman, on the `turn`th turn of the
/// Historical schedule (2-4).
fn advance_goal(state: &GameState, unit: &UnitPlacement, turn: u8) -> Option<Goal> {
    let identity = unit.profile.identity;
    let kerreri = HISTORICAL_KERRERI_UNITS.contains(&unit.id);
    // The advance. A unit the enemy has closed on halts and fights where it
    // stands -- in the formation, with the stacks round it to shoot down
    // whatever comes on (§6.7), not out at the attackers.
    let near_enemy = state.units.iter().any(|u| {
        u.profile.identity.owner() == Player::Dervish
            && u.profile.identity != UnitIdentity::DervishFort
            && u.position.distance(unit.position) <= ENGAGE_RANGE
    });
    if near_enemy {
        return Some(Goal::Hex(unit.position));
    }
    let goal = match identity {
        UnitIdentity::AngloEgyptianInfantry { .. } => {
            let brigade = identity.brigade()?;
            Goal::March(slot(state, brigade, turn))
        }
        // The 21st Lancers ride at the head of the column, by the river.
        UnitIdentity::AngloEgyptianCavalry if !kerreri => {
            let (q, row) = march_point(turn);
            Goal::March(on_land(state, printed(q + 1, row + 1)))
        }
        // The Egyptian cavalry and the Camel Corps cover the right rear,
        // between Macdonald and the rearguard: mounted troops caught alone
        // by the tribes are lost (the camels could not outrun them on
        // Kerreri).
        UnitIdentity::AngloEgyptianCavalry | UnitIdentity::AngloEgyptianCamelCorps => {
            let (q, row) = march_point(turn);
            Goal::March(on_land(state, printed(q - 6, row - 6)))
        }
        // Guns and Maxims march with the British brigades, a little behind.
        _ => {
            let (q, row) = march_point(turn);
            Goal::Band {
                target: on_land(state, printed(q - 1, row)),
                standoff: 2,
            }
        }
    };
    Some(goal)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The leaders' colours (§9.212) decide the flags.
    #[test]
    fn every_tribe_in_the_historical_scenario_has_a_flag() {
        for tribe in [
            DervishTribe::Taiasha,
            DervishTribe::Baggara,
            DervishTribe::Jaalin,
            DervishTribe::Hadendowa,
            DervishTribe::Mulazmin,
            DervishTribe::Jehadia,
            DervishTribe::Danagla,
            DervishTribe::Kehena,
            DervishTribe::Degheim,
        ] {
            assert!(leader_of(&UnitIdentity::DervishTribal { tribe }).is_some());
        }
    }

    /// The printed rows undo the axial skew: a column's neighbours one row
    /// down are the same printed column one row lower.
    #[test]
    fn printed_rows_follow_the_page() {
        for q in -3..4 {
            let a = printed(q, 2);
            let b = printed(q, 3);
            assert!(a.is_adjacent_to(b));
        }
    }
}

//! The open end of Khartoum's wall (Fall of Khartoum, §9.3).
//!
//! The wall runs from the Blue Nile round the south of the city and stops
//! short of the White Nile: on the map, as in January 1885 when the river
//! had fallen and left a strip of dry ground between the last stretch of
//! wall and the water, there is a gap no gate guards. The Mahdists came in
//! that way on the night of the 26th. This module finds the gap from the
//! board -- the one end of the wall line that meets neither another wall
//! hexside nor the river -- for both commanders: the Khalifa's assault may
//! take it, the garrison must hold it like a gate (§5.23).

use omdurman_rules::effects::GameState;
use omdurman_types::{HexCoord, HexsideKind};

/// The gap between the wall's open end and the river.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WallGap {
    /// The hexes across the gap on the city side, from the wall's end to
    /// the river bank: the line the garrison holds.
    pub line: Vec<HexCoord>,
    /// The hex just outside the wall's end: the assault's way in.
    pub approach: HexCoord,
}

fn walled(state: &GameState, a: HexCoord, b: HexCoord) -> bool {
    matches!(
        state.hexside_effective(a, b),
        Some(HexsideKind::Wall | HexsideKind::Gate | HexsideKind::Breach)
    )
}

fn land(state: &GameState, h: HexCoord) -> bool {
    state.board.terrain.contains_key(&h) && !state.board.is_nile(h)
}

/// The gap at the wall's open end, if the board's wall has one: a wall
/// hexside one of whose corners touches a land hex across no other wall
/// hexside. `palace` tells the city side from the outside.
pub fn wall_gap(state: &GameState, palace: HexCoord) -> Option<WallGap> {
    for (hr, _) in &state.board.hexsides {
        let (a, b) = (hr.a, hr.b);
        if !walled(state, a, b) {
            continue;
        }
        for c in a.neighbors() {
            if !b.is_adjacent_to(c) || !land(state, c) {
                continue;
            }
            if walled(state, a, c) || walled(state, b, c) {
                continue;
            }
            // `c` stands at the wall's end. Of `a` and `b`, the one nearer
            // the palace is the city side's end of the line.
            let inside = if palace.distance(a) <= palace.distance(b) {
                a
            } else {
                b
            };
            let line = line_to_river(state, inside, c, palace)?;
            return Some(WallGap { line, approach: c });
        }
    }
    None
}

/// The shortest chain of land hexes from `start` to a hex on the river
/// bank, keeping off `outside` (the approach side) and level with `start`
/// as seen from the palace: the line across the gap. Ties keep the board
/// order of the neighbours (reproducible).
fn line_to_river(
    state: &GameState,
    start: HexCoord,
    outside: HexCoord,
    palace: HexCoord,
) -> Option<Vec<HexCoord>> {
    use std::collections::{HashMap, VecDeque};
    let on_bank = |h: HexCoord| h.neighbors().iter().any(|&n| state.board.is_nile(n));
    let mut prev: HashMap<HexCoord, HexCoord> = HashMap::new();
    let mut queue = VecDeque::from([start]);
    let mut seen = vec![start, outside];
    while let Some(h) = queue.pop_front() {
        if on_bank(h) {
            let mut line = vec![h];
            while let Some(&p) = prev.get(line.last().expect("non-empty")) {
                line.push(p);
            }
            line.reverse();
            return Some(line);
        }
        for n in h.neighbors() {
            // Across the gap, not round behind the wall or out through it.
            if seen.contains(&n) || !land(state, n) || walled(state, h, n) {
                continue;
            }
            // On the city side of the end: off the approach's doorstep,
            // and near enough to be the gap.
            if n.is_adjacent_to(outside)
                || n.distance(outside) > 3
                || n.distance(palace) > start.distance(palace) + 1
            {
                continue;
            }
            seen.push(n);
            prev.insert(n, h);
            queue.push_back(n);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use omdurman_types::{Location, Scenario};

    /// The printed map: the wall's south-western end at the White Nile
    /// leaves a four-hex strip of dry ground.
    #[test]
    fn the_fall_of_khartoum_wall_leaves_the_white_nile_gap() {
        let board = crate::playthrough::board_for_scenario(Scenario::FallOfKhartoum);
        let state = GameState::with_board(Scenario::FallOfKhartoum, board);
        let palace = state.board.hex_of_location(Location::Palace).unwrap();
        let gap = wall_gap(&state, palace).expect("the wall has an open end");
        assert_eq!(gap.approach, HexCoord::new(10, 12));
        assert_eq!(
            gap.line,
            vec![
                HexCoord::new(10, 11),
                HexCoord::new(9, 10),
                HexCoord::new(8, 10),
                HexCoord::new(7, 9)
            ]
        );
    }
}

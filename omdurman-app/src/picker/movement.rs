//! Plot-time movement rules: passability, terrain costs, MP budgets — the
//! app-side mirror of the engine's movement checks (§5.11/§5.22/§5.24/§9.233).

use super::*;

/// App-side §5.22 gate: land units may not enter Nile hexes; gunboats may
/// only enter Nile hexes.
fn terrain_passable(terrain: Terrain, is_boat: bool) -> bool {
    if is_boat {
        terrain.is_nile()
    } else {
        terrain.passable_by_land()
    }
}

/// Whether a unit may occupy `coord`. Off-map coordinates (those not present
/// in `game_map.hexes`, which is clipped to the active overlay) are never
/// valid -- earlier code allowed land units to be placed off-map because the
/// map wasn't guaranteed loaded; late-joiners now replay the event log from
/// compiled board data, guaranteeing the map is populated before placement.
pub(crate) fn coord_passable(game_map: &GameMap, coord: HexCoord, is_boat: bool) -> bool {
    game_map
        .hexes
        .get(&coord)
        .is_some_and(|h| terrain_passable(h.terrain, is_boat))
}

/// Movement points required to step `from` -> `coord` (§5.11, Terrain
/// Effects Chart): the shared `terrain_chart::land_step_cost` rule -- the
/// entered terrain (1 along a road link) plus the crossed hexside (Khor +5,
/// Crest +1, gate/breach +1, §9.233 Zariba end +2). With a game state the
/// engine prices the step itself (it knows the breaches); the bare map is
/// the editor/no-game fallback. Gunboats pay a flat 1 MP per entered Nile
/// hex (§5.24). Returns 0 if the step is off-map or closed to the mover
/// (callers should check passability separately).
pub(crate) fn floor_movement_cost(
    game_map: &GameMap,
    from: HexCoord,
    coord: HexCoord,
    is_boat: bool,
    game_state: Option<&omdurman_rules::effects::GameState>,
) -> i16 {
    if is_boat {
        return if coord_passable(game_map, coord, true) {
            1
        } else {
            0
        };
    }
    if !coord_passable(game_map, coord, false) {
        return 0;
    }
    if let Some(gs) = game_state {
        return i16::try_from(gs.land_step_cost(from, coord))
            .ok()
            .filter(|&c| c < i16::MAX)
            .unwrap_or(0);
    }
    let Some(tile) = game_map.hexes.get(&coord) else {
        return 0;
    };
    omdurman_rules::terrain_chart::land_step_cost(
        tile.terrain,
        game_map.roads.contains(&HexsideRef::new(from, coord)),
        game_map.hexside_between(from, coord),
    )
    .unwrap_or(0)
}

/// §5.24: a gunboat's whole turn is capped at the upstream allowance once any
/// step of the pending path (or an earlier move this turn) went upstream. A
/// scalar `remaining_mp` budget can't express that cap for a mixed route, so
/// the click gate re-derives it exactly like the engine's `can_move_gunboat`:
/// spent so far + plotted cost so far + this leg must fit the allowance that
/// binds once this leg is on the path. Land movers always pass (their scalar
/// budget already matches the engine's cumulative check).
pub(crate) fn gunboat_cap_ok(
    game_state: Option<&crate::GameStateResource>,
    placed: &PlacedUnit,
    movement_path: &MovementPath,
    from: HexCoord,
    to: HexCoord,
    leg_cost: i16,
) -> bool {
    let Some(uid) = placed.unit_id else {
        return true;
    };
    let Some(gs) = game_state else {
        return true;
    };
    let Some(unit) = gs.0.find_unit(uid) else {
        return true;
    };
    let omdurman_rules::UnitMovement::Gunboat(g) = unit.profile.movement else {
        return true;
    };
    let upstream = |a: HexCoord, b: HexCoord| {
        gs.0.board.step_direction(a, b) == Some(omdurman_rules::board::StepDirection::Upstream)
    };
    let leg_upstream = upstream(from, to);
    let pending_upstream = movement_path.legs.iter().any(|&(f, t)| upstream(f, t));
    let sticky = gs.0.gunboats_upstream_this_turn.contains(&uid);
    if !leg_upstream && !pending_upstream && !sticky {
        // Pure-downstream route: the downstream allowance binds, and
        // `remaining_mp` already encodes it.
        return true;
    }
    gs.0.mp_spent(uid) + movement_path.cost_so_far + leg_cost <= g.upstream.value() as i16
}

/// Remaining movement points for a placed unit this turn, from the rules
/// engine: full (night-adjusted) allowance minus what the unit already spent
/// (§5.11/§5.12). Units with no rules identity, or in a session with no game
/// state (editor), are treated as unconstrained (99). This is the budget the
/// picker plots movement against for both single and stack selection.
pub(crate) fn unit_remaining_mp(
    game_state: Option<&crate::GameStateResource>,
    placed: &PlacedUnit,
) -> i16 {
    if let Some(uid) = placed.unit_id
        && let Some(gs) = game_state
        && let Some(unit) = gs.0.find_unit(uid)
    {
        match unit.profile.movement {
            omdurman_rules::UnitMovement::Land(a) => {
                let effective = omdurman_rules::effective_movement_at_night(
                    a,
                    unit.profile.identity.owner(),
                    gs.0.day_night,
                );
                (effective.value() as i16 - gs.0.mp_spent(uid)).max(0)
            }
            omdurman_rules::UnitMovement::Gunboat(g) => {
                let spent = gs.0.mp_spent(uid);
                // §5.24: once the boat has taken an upstream step this turn,
                // the upstream allowance is the cap for the *whole* turn
                // (sticky) -- even for otherwise downstream moves.
                let allowance = if gs.0.gunboats_upstream_this_turn.contains(&uid) {
                    g.upstream.value() as i16
                } else {
                    g.upstream.value().max(g.downstream.value()) as i16
                };
                (allowance - spent).max(0)
            }
            _ => 99,
        }
    } else {
        99
    }
}

/// Borrowed context for resolving a click while a unit is selected.
///
/// As with [`PlacingClick`], the map query is passed to
/// [`handle`](Self::handle) rather than stored, so the invariant `Query`
/// lifetimes never couple to the struct's `Commands` borrow.
/// Shared facts about one proposed movement-path leg. Computed identically for
/// single-unit ([`SelectedClick`]) and whole-stack ([`SelectedStackClick`])
/// path building; only the affordability predicate differs between the two.
pub(crate) struct MovementLegCheck {
    /// A unit may not move into an enemy-occupied hex (that's melee / advance
    /// after combat, not movement) -- except the §9.346 palace waiver: a
    /// Dervish unit may move *onto* the Palace even though GORDON occupies it.
    pub(crate) enemy_occupied: bool,
    /// Engine `check_stacking` view of the leg's destination — whether the
    /// mover may *finish* there (§5.51 cap, gunboat exclusivity, tribe-mix,
    /// leader-command rules). Deliberately NOT a leg blocker: a path may pass
    /// *through* a friendly hex even when a stop there would exceed the cap
    /// (§5.51 — the stacking limit applies at the end of the move). The commit
    /// gate re-checks it against the plotted final destination. `true` when
    /// there is no engine state to consult.
    pub(crate) stacking_ok: bool,
    /// §5.43: this leg's destination is in an enemy ZOC, so the builder must
    /// force a stop (no further legs this turn). A unit that *began* in a ZOC
    /// may still move out, so only the destination matters.
    pub(crate) entering_enemy_zoc: bool,
    /// Adjacency is checked against the *planned* current position
    /// (`start_coord`), not the unit's original placed coord, so multi-leg
    /// path building works correctly.
    pub(crate) adjacent: bool,
    pub(crate) passable: bool,
    /// Terrain cost of entering `coord` (0 when off-map, impassable, or not
    /// adjacent; callers require `cost > 0` for affordability).
    pub(crate) cost: i16,
}

impl MovementLegCheck {
    /// Whether the proposed leg is legal to *move into* as part of a plotted
    /// path. §5.51 lets a path pass through friendly-occupied hexes even when
    /// the transient occupancy exceeds the stacking cap, so `stacking_ok` is
    /// deliberately not part of this predicate — the cap binds where the mover
    /// ends its move (the commit gate plus the engine's apply-time re-check).
    /// `affordable` is the caller's budget predicate (single-unit vs whole
    /// stack); `forced_stop` is the sticky §5.43 enemy-ZOC stop.
    pub(crate) fn accepted(&self, affordable: bool, forced_stop: bool) -> bool {
        self.adjacent && !self.enemy_occupied && self.passable && affordable && !forced_stop
    }
}

/// Whether a wall / Zariba hexside blocks the step `start -> coord` (a
/// §6.63 breach reopens it, which only the engine state knows).
pub(crate) fn hexside_blocks_step(
    game_map: &GameMap,
    start: HexCoord,
    coord: HexCoord,
    game_state: Option<&crate::GameStateResource>,
) -> bool {
    match game_state {
        Some(gs) => {
            gs.0.hexside_effective_is(start, coord, omdurman_types::HexsideKind::blocks_movement)
        }
        None => game_map
            .hexside_between(start, coord)
            .is_some_and(omdurman_types::HexsideKind::blocks_movement),
    }
}

/// Compute the shared leg facts for a mover represented by `placed` (the stack
/// variant passes its first unit; the engine re-validates every unit at
/// commit). `uid` is the mover's rules identity, when one exists.
pub(crate) fn movement_leg_check(
    game_map: &GameMap,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    placed: &PlacedUnit,
    start_coord: HexCoord,
    coord: HexCoord,
    game_state: Option<&crate::GameStateResource>,
) -> MovementLegCheck {
    let mover_owner = omdurman_rules::unit_profiles::section_owner(placed.section_name);

    // §9.346: "passing through or occupying the palace hex" is how GORDON is
    // eliminated, so the enemy-occupation gate is waived for the Palace (the
    // faction gate upstream already ensures only the side whose turn it is can
    // reach here; the engine resolves GORDON's death for a Dervish occupant).
    let dest_is_palace = game_map.hexes.get(&coord).is_some_and(|h| {
        h.name
            .as_deref()
            .and_then(omdurman_types::Location::from_tile_name)
            == Some(omdurman_types::Location::Palace)
    });
    // §5.51: a path may pass *through* friendly-occupied hexes at no extra
    // movement points; the stacking cap binds only where the move **ends**. So
    // enemy occupancy is the only occupancy-based wall here — `stacking_ok`
    // decides the finish, not the pass-through.
    let enemy_occupied = !dest_is_palace
        && mover_owner.is_some()
        && placed_units.iter().any(|(_, u)| {
            u.coord == coord
                && omdurman_rules::unit_profiles::section_owner(u.section_name) != mover_owner
        });
    // With no engine state (editor session), defer to commit-time validation.
    let (stacking_ok, entering_enemy_zoc) = match (placed.unit_id, game_state) {
        (Some(uid), Some(gs)) if let Some(mover) = gs.0.find_unit(uid) => (
            gs.0.check_stacking(mover, coord).is_ok(),
            gs.0.hex_in_enemy_zoc(coord, mover.profile.identity.owner(), mover.profile.kind),
        ),
        _ => (true, false),
    };
    let adjacent = start_coord.neighbors().contains(&coord);
    let hexside_blocks = hexside_blocks_step(game_map, start_coord, coord, game_state);
    let passable = coord_passable(game_map, coord, placed.is_boat) && !hexside_blocks;
    let cost = if adjacent {
        floor_movement_cost(
            game_map,
            start_coord,
            coord,
            placed.is_boat,
            game_state.map(|gs| &gs.0),
        )
    } else {
        0
    };
    MovementLegCheck {
        enemy_occupied,
        stacking_ok,
        entering_enemy_zoc,
        adjacent,
        passable,
        cost,
    }
}

/// The cheapest chain of legs from `start` to `goal` that the plot-time leg
/// checks accept -- so clicking a distant hex plots a whole route instead of
/// demanding one click per hex. Only through passable, non-enemy hexes; a
/// hex entering an enemy ZOC ends the route there (§5.43), so it is only
/// usable as the goal; stacking binds at the goal alone (§5.51). `budget` is
/// the largest remaining MP of the mover(s). Returns the hexes after `start`,
/// or `None` if the goal is out of reach.
pub(crate) fn auto_route(
    game_map: &GameMap,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    placed: &PlacedUnit,
    start: HexCoord,
    goal: HexCoord,
    budget: i16,
    game_state: Option<&crate::GameStateResource>,
) -> Option<Vec<HexCoord>> {
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, HashMap};
    let mut best: HashMap<HexCoord, i16> = HashMap::from([(start, 0)]);
    let mut prev: HashMap<HexCoord, HexCoord> = HashMap::new();
    let mut heap = BinaryHeap::from([Reverse((0i16, start.q, start.r))]);
    while let Some(Reverse((cost, q, r))) = heap.pop() {
        let hex = HexCoord::new(q, r);
        if hex == goal {
            let mut route = vec![goal];
            while let Some(&p) = prev.get(route.last()?) {
                if p == start {
                    break;
                }
                route.push(p);
            }
            route.reverse();
            return Some(route);
        }
        if best.get(&hex).is_some_and(|&b| b < cost) {
            continue;
        }
        for next in hex.neighbors() {
            if !game_map.hexes.contains_key(&next) {
                continue;
            }
            let leg = movement_leg_check(game_map, placed_units, placed, hex, next, game_state);
            if leg.enemy_occupied || !leg.passable || leg.cost <= 0 {
                continue;
            }
            // A ZOC hex may end the route but not be passed through; a full
            // friendly stack may be passed through but not ended in.
            if (next != goal && leg.entering_enemy_zoc) || (next == goal && !leg.stacking_ok) {
                continue;
            }
            let total = cost + leg.cost;
            if total > budget || best.get(&next).is_some_and(|&b| b <= total) {
                continue;
            }
            best.insert(next, total);
            prev.insert(next, hex);
            heap.push(Reverse((total, next.q, next.r)));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn leg(
        enemy_occupied: bool,
        stacking_ok: bool,
        adjacent: bool,
        passable: bool,
    ) -> MovementLegCheck {
        MovementLegCheck {
            enemy_occupied,
            stacking_ok,
            entering_enemy_zoc: false,
            adjacent,
            passable,
            cost: 1,
        }
    }

    /// The stacking cap binds only where the move *ends*, so a path may be
    /// plotted through a friendly-occupied hex even when a stop there would
    /// exceed the cap. `stacking_ok == false` must not block the leg.
    #[test]
    fn friendly_pass_through_is_accepted_despite_stacking_cap() {
        let over_cap = leg(false, false, true, true);
        assert!(
            over_cap.accepted(true, false),
            "a friendly over-cap hex is a legal pass-through leg"
        );
    }

    /// The genuine movement blockers still reject the leg.
    #[test]
    fn genuine_blocks_reject_the_leg() {
        assert!(
            !leg(true, true, true, true).accepted(true, false),
            "enemy-occupied"
        );
        assert!(
            !leg(false, true, false, true).accepted(true, false),
            "non-adjacent"
        );
        assert!(
            !leg(false, true, true, false).accepted(true, false),
            "impassable"
        );
        assert!(
            !leg(false, true, true, true).accepted(false, false),
            "unaffordable"
        );
        assert!(
            !leg(false, true, true, true).accepted(true, true),
            "§5.43 forced stop"
        );
    }

    // -- Plot-time cost parity with the engine ------------------------------

    /// A two-hex clear map with an optional hexside annotation between them.
    fn two_hex_map(hexside: Option<omdurman_types::HexsideKind>) -> GameMap {
        let mut map = GameMap::default();
        for q in 0..=1 {
            map.hexes.insert(
                HexCoord::new(q, 0),
                omdurman_types::HexData {
                    terrain: Terrain::Clear {
                        road: Default::default(),
                    },
                    location: None,
                    name: None,
                    setup_letter: None,
                    is_scattergram: false,
                    named_area: None,
                },
            );
        }
        if let Some(kind) = hexside {
            map.hexsides.insert(
                HexsideRef::new(HexCoord::new(0, 0), HexCoord::new(1, 0)),
                kind,
            );
        }
        map
    }

    /// §9.233: crossing a Zariba/trench end hexside costs +2 MP. The plot-time
    /// leg cost must match the engine's `movement_cost_for`, or the app plots
    /// routes the engine rejects at commit (the plotted move then silently
    /// does nothing).
    #[test]
    fn trench_end_crossing_costs_two_extra_mp() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(1, 0);
        let map = two_hex_map(Some(omdurman_types::HexsideKind::ZaribaTrenchEndA));
        assert_eq!(
            floor_movement_cost(&map, a, b, false, None),
            3,
            "clear (1) + trench-end surcharge (2)"
        );
        // The reverse crossing pays the surcharge too (the end hexside is
        // bidirectional).
        assert_eq!(floor_movement_cost(&map, b, a, false, None), 3);
        // An ordinary hexside crossing has no surcharge.
        let plain = two_hex_map(None);
        assert_eq!(floor_movement_cost(&plain, a, b, false, None), 1);
    }

    /// A campaign state with one Dervish gunboat (upstream 10 / downstream 16)
    /// on a straight East-flowing Nile row.
    fn gunboat_state() -> (
        crate::GameStateResource,
        UnitId,
        omdurman_types::SectionName,
    ) {
        use omdurman_rules::board::BoardInfo;
        use omdurman_rules::effects::GameState;
        use omdurman_rules::{
            GunboatId, GunboatMovement, MovementAllowance, UnitIdentity, UnitMovement, WeaponClass,
        };
        use std::sync::Arc;

        let mut state = GameState::new(Scenario::Campaign);
        let mut board = BoardInfo::default();
        for q in 0..=14 {
            board.terrain.insert(
                HexCoord::new(q, 0),
                Terrain::Nile {
                    direction: omdurman_types::HexDirection::East,
                },
            );
        }
        state.board = Arc::new(board);
        state.phase = omdurman_rules::Phase::Movement;
        state.active_player = omdurman_types::Player::Dervish;
        let id = state.alloc_unit_id();
        state.units.push(UnitPlacement {
            id,
            position: HexCoord::new(3, 0),
            profile: omdurman_rules::UnitProfile {
                kind: omdurman_types::UnitKind::Gunboat {
                    fire: 0,
                    upstream: 0,
                    downstream: 0,
                },
                identity: UnitIdentity::DervishGunboat(GunboatId::DervishGunboat(1)),
                weapon: WeaponClass::Artillery,
                fire: None,
                melee: None,
                movement: UnitMovement::Gunboat(GunboatMovement {
                    upstream: MovementAllowance::Ten,
                    downstream: MovementAllowance::Sixteen,
                }),
            },
            state: UnitState::default(),
        });
        (
            crate::GameStateResource(state),
            id,
            omdurman_types::SectionName::Hadendowa,
        )
    }

    fn boat_placement(
        coord: HexCoord,
        section: omdurman_types::SectionName,
        unit_id: Option<UnitId>,
    ) -> PlacedUnit {
        PlacedUnit {
            coord,
            section_name: section,
            col: 0,
            row: 0,
            is_boat: true,
            unit_id,
            disrupted: false,
        }
    }

    /// §5.24: the §5.24 sticky upstream cap and the whole-path upstream
    /// allowance must bind at *plot* time. The scalar `remaining_mp` budget
    /// (max of up/down) can't express them, so `gunboat_cap_ok` re-derives the
    /// engine's rule per leg; a plotted route the engine would reject must
    /// never be confirmable.
    #[test]
    fn gunboat_cap_gate_mirrors_the_engine() {
        let (mut gs, id, section) = gunboat_state();
        let placed = boat_placement(HexCoord::new(3, 0), section, Some(id));
        let upstream = |q: i32| HexCoord::new(q - 1, 0);
        let downstream = |q: i32| HexCoord::new(q + 1, 0);

        // A purely-downstream leg passes: the downstream allowance binds.
        let empty = MovementPath::default();
        assert!(gunboat_cap_ok(
            Some(&gs),
            &placed,
            &empty,
            HexCoord::new(3, 0),
            downstream(3),
            1
        ));

        // A long all-downstream pending path plus one upstream leg: the whole
        // turn is capped at the upstream allowance (10). 8 plotted + 1 = 9
        // fits, and 9 plotted + 1 = 10 exactly meets it; 10 plotted + 1 = 11
        // does not.
        let mut pending = MovementPath::default();
        for q in 3..11 {
            pending.legs.push((HexCoord::new(q, 0), downstream(q)));
        }
        pending.cost_so_far = 8;
        assert!(gunboat_cap_ok(
            Some(&gs),
            &placed,
            &pending,
            HexCoord::new(11, 0),
            upstream(11),
            1
        ));
        pending.legs.push((HexCoord::new(11, 0), downstream(11)));
        pending.cost_so_far = 9;
        assert!(gunboat_cap_ok(
            Some(&gs),
            &placed,
            &pending,
            HexCoord::new(12, 0),
            upstream(12),
            1
        ));
        pending.legs.push((HexCoord::new(12, 0), downstream(12)));
        pending.cost_so_far = 10;
        assert!(!gunboat_cap_ok(
            Some(&gs),
            &placed,
            &pending,
            HexCoord::new(13, 0),
            upstream(13),
            1
        ));

        // Sticky cap: after an upstream move this turn, even an all-downstream
        // leg is capped at the upstream allowance (10).
        gs.0.gunboats_upstream_this_turn.push(id);
        let fresh = MovementPath::default();
        assert!(gunboat_cap_ok(
            Some(&gs),
            &placed,
            &fresh,
            HexCoord::new(3, 0),
            downstream(3),
            9
        ));
        assert!(!gunboat_cap_ok(
            Some(&gs),
            &placed,
            &fresh,
            HexCoord::new(3, 0),
            downstream(3),
            11
        ));
    }

    /// §5.24: a boat that went upstream earlier this turn has the *upstream*
    /// allowance as its remaining budget (not the larger downstream one), so
    /// re-selecting it must not offer more hexes than the engine will accept.
    #[test]
    fn remaining_mp_respects_the_sticky_upstream_cap() {
        let (mut gs, id, section) = gunboat_state();
        let placed = boat_placement(HexCoord::new(3, 0), section, Some(id));
        assert_eq!(unit_remaining_mp(Some(&gs), &placed), 16);
        gs.0.gunboats_upstream_this_turn.push(id);
        assert_eq!(unit_remaining_mp(Some(&gs), &placed), 10);
    }
}

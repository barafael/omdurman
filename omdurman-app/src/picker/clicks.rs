//! Board click handling: the place / select / move state machine.

use super::*;
use crate::board_click::PickerClick;
use crate::hotkeys::PickerCommand;
use bevy::ecs::message::MessageReader;

/// Max seconds between two left-clicks on the same hex for them to count as a
/// double-click (select-the-whole-stack, movement phase).
const DOUBLE_CLICK_SECS: f64 = 0.35;

/// The spatial facts of one board click: the ground-plane hit point, the
/// clicked hex's centre in world space, and the stack spread used to pick
/// among stacked counters.
struct ClickGeometry {
    hit: Vec3,
    center: Vec3,
    stack_spread: f32,
    hex_size: f32,
}

/// Bundles the picker-specific resources, queries, and command buffers so
/// [`handle_picker_clicks`] and [`unit_picker_ui`] stay under the
/// system-parameter limit.  Resources that are also consumed by *other* systems
/// (e.g. `HexLayout`) are included here because they are logically
/// part of the picker's map-interaction domain; those systems continue to take
/// the individual `Res`s.
#[derive(bevy::ecs::system::SystemParam)]
pub struct PickerContext<'w, 's> {
    pub picker: ResMut<'w, UnitPicker>,
    pub state: ResMut<'w, PickerState>,
    pub movement_path: ResMut<'w, MovementPath>,
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
    pub placed_units: Query<'w, 's, (Entity, &'static PlacedUnit)>,
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub action_writer: MessageWriter<'w, events::LocalAction>,
}

/// Reads the [`PickerClick`]s routed by
/// [`route_board_clicks`](crate::board_click::route_board_clicks) — the
/// router has already decided the picker owns them — and drives the place /
/// select / plot state machine.
pub fn handle_picker_clicks(
    mut clicks: MessageReader<PickerClick>,
    mut picker_ctx: PickerContext,
    game_state: Res<crate::GameStateResource>,
    peers: crate::peers::Peers,
    time: Res<Time>,
    // A refused click: the engine's reason to the feed, a flash on the hex.
    mut dispatches: Option<ResMut<crate::dispatch::Dispatches>>,
    mut fx: MessageWriter<crate::fx::FxRequest>,
    mut last_click: Local<Option<(f64, HexCoord)>>,
) {
    let game_state = &game_state;
    let now = time.elapsed_secs_f64();
    for &click in clicks.read() {
        let refusal = picker_click(
            click,
            &mut picker_ctx,
            game_state,
            &peers,
            now,
            &mut last_click,
        );
        if let Some(reason) = refusal {
            // The reason goes to the feed; the flash says where.
            fx.write(crate::fx::FxRequest::Refused { hex: click.hex });
            if let Some(dispatches) = dispatches.as_deref_mut() {
                dispatches.refuse(crate::submit::REFUSED_HEADER, reason);
            }
        }
        if let Some(notice) = picker_ctx.movement_path.notice.take()
            && let Some(dispatches) = dispatches.as_deref_mut()
        {
            dispatches.push("Stack Order", notice);
        }
    }
}

/// One routed picker click (see [`handle_picker_clicks`]).
fn picker_click(
    PickerClick {
        hex: coord,
        hit,
        pressed,
        released,
    }: PickerClick,
    picker_ctx: &mut PickerContext,
    game_state: &crate::GameStateResource,
    peers: &crate::peers::Peers,
    now: f64,
    last_click: &mut Option<(f64, HexCoord)>,
) -> Option<String> {
    // §turn-order: a unit may only be moved on its owner's turn: gate
    // interactive movement on the local player being the rules engine's
    // active player (the Idle arm / move path below). Placement during
    // set-up is not gated.
    // (The click router already drops clicks outside the seat's phase unless
    // a counter is in hand; this stays as the picker's own backstop.)
    let may_move = peers.may_act_now(&game_state.0);

    // In bound multiplayer a player may only pick up their own faction's units;
    // an unbound session / single-seat (no faction bindings) may move
    // either side. `may_move` already gates *that it's the right turn*.
    let restrict_to = if !peers.any_assigned() {
        None
    } else {
        peers.local()
    };
    // §1.1 multi-player commands: with scopes assigned at StartGame, a unit
    // another member commands may not be picked up. Communal units (no
    // scope claims them) are every member's to act on.

    let origin = picker_ctx
        .layout
        .adjusted_origin(&picker_ctx.overlay.params);
    // World-space centre of the clicked hex -- used to resolve which counter in
    // a stack is under the cursor (stacks fan out around the centre).
    let center = hex_world_pos(coord, origin, &picker_ctx.overlay.params);
    let stack_spread = 0.34 * picker_ctx.overlay.params.hex_size;
    let click = ClickGeometry {
        hit,
        center,
        stack_spread,
        hex_size: picker_ctx.overlay.params.hex_size,
    };

    // During Setup, clicking a placed unit focuses it (blue/orange outline) so
    // the player can hit Del to return it to the picker. This short-circuits
    // the Idle/Selected state machine for placed-unit clicks, so it works
    // whether or not another unit is already focused (focus switch). Bound
    // game: don't focus an enemy counter. Skipped while a unit is in hand
    // (`Placing`) so the player can still stack onto an occupied hex.
    if pressed
        && !matches!(*picker_ctx.state, PickerState::Placing { .. })
        && (matches!(game_state.0.phase, omdurman_rules::Phase::Setup))
    {
        // Pick the specific counter under the cursor (a stack fans out, so the
        // nearest-by-rendered-position is the right one, not just the first).
        if let Some(entity) = nearest_placed_unit_at(
            &picker_ctx.placed_units,
            coord,
            center,
            hit,
            stack_spread,
            picker_ctx.overlay.params.hex_size,
        ) && let Ok((_, placed)) = picker_ctx.placed_units.get(entity)
        {
            let owner = omdurman_rules::unit_profiles::section_owner(placed.section_name);
            if restrict_to.is_some_and(|f| owner != Some(f)) {
                return None; // not your unit
            }
            // Not your command either (§1.1): skip to the next counter of the
            // stack rather than eating the click, so a mixed stack stays
            // fully operable by its owners.
            if let Some(identity) = omdurman_rules::unit_profiles::identity_for_counter(
                placed.section_name,
                placed.col,
                placed.row,
            ) && !peers.scope_allows(&identity)
            {
                return None; // another member's command
            }
            picker_ctx.commands.entity(entity).try_insert(Selected);
            *picker_ctx.state = PickerState::Selected {
                source: entity,
                start_coord: coord,
                remaining_mp: 0,
                forced_stop: false,
            };
            return None;
        }
    }

    // Double-click (same hex, within `DOUBLE_CLICK_SECS`) selects the whole
    // stack, *anywhere on the hex* — detection is hex-based, so neither click
    // of the pair needs to land on a counter symbol. Movement/Setup get the
    // group-move stack; a fire sub-phase or Melee gets the combat tile
    // ([`select_combat_tile`]). The first click of the pair has already run
    // the single-unit selection (or missed, harmlessly), so this override
    // must clear whatever it left behind — `select_combat_tile` /
    // `handle_stack_double_click` do.
    //
    // The early `return` is what keeps the tile selection alive: the release
    // of this very press must not run the tile arm's dismiss logic (the old
    // fire double-click selected on the second press and dismissed on its
    // release — a selection that lived for one frame).
    let double_click = if pressed {
        let is_dc = last_click
            .as_ref()
            .is_some_and(|&(t, c)| now - t <= DOUBLE_CLICK_SECS && c == coord);
        *last_click = Some((now, coord));
        is_dc
    } else {
        false
    };

    if pressed
        && double_click
        && may_move
        && !matches!(&*picker_ctx.state, PickerState::Placing { .. })
    {
        match game_state.0.phase {
            omdurman_rules::Phase::OffensiveFire(_)
            | omdurman_rules::Phase::DefensiveFire(_)
            | omdurman_rules::Phase::Melee => {
                select_combat_tile(
                    &mut picker_ctx.state,
                    &mut picker_ctx.commands,
                    &picker_ctx.placed_units,
                    game_state,
                    coord,
                    restrict_to,
                    peers,
                );
            }
            // Movement and Setup keep the group-move stack selection (a
            // double-click elsewhere is a no-op).
            omdurman_rules::Phase::Movement | omdurman_rules::Phase::Setup => {
                handle_stack_double_click(
                    &mut picker_ctx.state,
                    &mut picker_ctx.commands,
                    &picker_ctx.placed_units,
                    coord,
                    game_state,
                    restrict_to,
                    peers,
                    &mut picker_ctx.movement_path,
                );
            }
        }
        return None;
    }

    match ActiveSelection::snapshot(&picker_ctx.state) {
        // Selecting a unit to move is only meaningful on your own turn.
        ActiveSelection::Idle if may_move && pressed => {
            select_single_unit(
                &mut picker_ctx.state,
                &mut picker_ctx.commands,
                &picker_ctx.placed_units,
                coord,
                game_state,
                restrict_to,
                peers,
                &click,
            );
        }
        ActiveSelection::Idle => {}
        // A spectator (bound game, no faction) may never place units. The picker
        // panel is hidden for spectators (`unit_picker_ui` early-returns), which
        // is the normal way `Placing` is entered -- this arm is the state-machine
        // backstop for any other path into `Placing` (stale state carried across
        // a role change, a future input source), resetting it rather than
        // committing a placement.
        ActiveSelection::Placing { .. } if peers.is_spectator() => {
            *picker_ctx.state = PickerState::Idle;
        }
        // Outside deployment a placement is a move of the phase player's
        // (reinforcement entry): it waits its turn and the pause like any
        // other action.
        ActiveSelection::Placing { .. }
            if pressed
                && (!matches!(game_state.0.phase, omdurman_rules::Phase::Setup)
                    && !peers.may_act_now(&game_state.0)) =>
        {
            return Some(if peers.paused() {
                "The game is paused until the absent player returns.".to_string()
            } else {
                "Units may only enter on their own side's movement phase.".to_string()
            });
        }
        // (Deployment-zone and every other set-up refusal comes from the
        // engine through `place_click`, with its reason.)
        ActiveSelection::Placing {
            unit_idx,
            drag_drop,
        } => {
            match place_click(
                picker_ctx,
                origin,
                released && !pressed,
                unit_idx,
                drag_drop,
                coord,
                game_state,
            ) {
                Ok(Some(event)) => {
                    picker_ctx
                        .action_writer
                        .write(events::LocalAction { event });
                }
                Ok(None) => {}
                Err(reason) => return Some(reason),
            }
        }
        ActiveSelection::Single {
            source,
            start_coord,
            remaining_mp,
            forced_stop,
        } if (matches!(game_state.0.phase, omdurman_rules::Phase::Movement)) => {
            // A press on the selected counter's own hex, before any step is
            // plotted, picks within the stack: another counter under the
            // cursor, or -- on the selected one again -- the next in turn.
            if pressed
                && picker_ctx.movement_path.legs.is_empty()
                && cycle_stack_selection(
                    picker_ctx,
                    source,
                    coord,
                    &click,
                    game_state,
                    restrict_to,
                    peers,
                )
            {
                return None;
            }
            let mut plan = SingleRoute {
                remaining_mp,
                forced_stop,
            };
            // Legs only extend the path (committed on confirm); a refusal
            // is a destination out of reach.
            let refusal = single_click(
                picker_ctx,
                &mut plan,
                released,
                source,
                start_coord,
                coord,
                game_state,
            )
            .err();
            if matches!(&*picker_ctx.state, PickerState::Idle) {
                picker_ctx.commands.entity(source).remove::<Selected>();
            }
            if refusal.is_some() {
                return refusal;
            }
        }
        // Outside the movement phase a single-unit selection is an *action*
        // target, not a mover: the click router sends combat-phase releases
        // to the fire / melee / advance handlers, never here, so there is
        // nothing to plot -- keep the selection so the fire overlay stays
        // active (§6.41). A press on another friendly counter switches the
        // selection to it (as the tile selection does), so a player can pick
        // the next firer without cancelling first -- a stackmate of the
        // selected counter included (§6.15 lets a stack divide its fire, so
        // after one battalion is allocated the next must be reachable without
        // cancelling); a press on the selected counter itself keeps it. A hex
        // holding enemy counters stays target territory for the release.
        ActiveSelection::Single { source, .. } => {
            if pressed {
                let owner = picker_ctx.placed_units.get(source).ok().and_then(|(_, p)| {
                    omdurman_rules::unit_profiles::section_owner(p.section_name)
                });
                let holds_foreign = picker_ctx.placed_units.iter().any(|(_, u)| {
                    u.coord == coord
                        && owner.is_some_and(|o| {
                            omdurman_rules::unit_profiles::section_owner(u.section_name) != Some(o)
                        })
                });
                let same_counter = nearest_placed_unit_at(
                    &picker_ctx.placed_units,
                    coord,
                    click.center,
                    click.hit,
                    click.stack_spread,
                    click.hex_size,
                )
                .is_some_and(|entity| entity == source);
                // The selected counter clicked again: the next one of its
                // stack (a single click otherwise always took the top one).
                if same_counter {
                    cycle_stack_selection(
                        picker_ctx,
                        source,
                        coord,
                        &click,
                        game_state,
                        restrict_to,
                        peers,
                    );
                } else if !holds_foreign
                    && select_single_unit(
                        &mut picker_ctx.state,
                        &mut picker_ctx.commands,
                        &picker_ctx.placed_units,
                        coord,
                        game_state,
                        restrict_to,
                        peers,
                        &click,
                    )
                {
                    picker_ctx.commands.entity(source).remove::<Selected>();
                }
            }
        }
        ActiveSelection::Stack(sel)
            if (matches!(game_state.0.phase, omdurman_rules::Phase::Movement)) =>
        {
            // Group move: the whole stack follows one plotted path. Legs charge
            // only the units that can afford them; slower units are dropped at
            // their last affordable hex. Commit (Enter) splits the path into
            // per-unit prefix `MoveUnit` events.
            let mut plan = StackRoute {
                remaining_mp: sel.remaining_mp.clone(),
                initial_mp: sel.initial_mp.clone(),
                forced_stop: sel.forced_stop,
            };
            if let Err(reason) = stack_click(
                picker_ctx,
                &mut plan,
                released,
                &sel.sources,
                sel.start_coord,
                coord,
                game_state,
            ) {
                return Some(reason);
            }
        }
        // A stale movement stack in a non-movement phase: combat target
        // clicks are routed to the combat handlers, so nothing to plot.
        ActiveSelection::Stack(_) => {}
        // The combat tile selection (double-click in a fire sub-phase or
        // Melee). Only *presses* act (the click router routes combat-phase
        // releases to the combat handlers; acting on the selecting
        // double-click's release would tear the tile down the same frame it
        // was made). A press on the tile's own hex
        // dismisses it; a press on another hex switches to a single-counter
        // selection there when one is pickable, and otherwise *keeps* the
        // tile — combat target clicks pass through here on their press and
        // act only on release (routed to `handle_fire_allocation_click` /
        // `handle_melee_combat`), so any other press must not destroy the
        // aiming selection.
        ActiveSelection::Tile(sel) => {
            if pressed {
                // A hex holding a counter foreign to the tile is combat-target
                // territory, never re-selection territory: reserving it keeps
                // the release free for `handle_fire_allocation_click` /
                // `handle_melee_combat` (and in an unbound session the single
                // select would otherwise happily grab the enemy counter under
                // the cursor).
                let tile_owner = sel.sources.iter().find_map(|&e| {
                    picker_ctx.placed_units.get(e).ok().and_then(|(_, p)| {
                        omdurman_rules::unit_profiles::section_owner(p.section_name)
                    })
                });
                let holds_foreign = picker_ctx.placed_units.iter().any(|(_, u)| {
                    u.coord == coord
                        && tile_owner.is_some_and(|o| {
                            omdurman_rules::unit_profiles::section_owner(u.section_name) != Some(o)
                        })
                });
                if coord == sel.start_coord {
                    for source in &sel.sources {
                        picker_ctx.commands.entity(*source).remove::<Selected>();
                    }
                    *picker_ctx.state = PickerState::Idle;
                } else if !holds_foreign
                    && select_single_unit(
                        &mut picker_ctx.state,
                        &mut picker_ctx.commands,
                        &picker_ctx.placed_units,
                        coord,
                        game_state,
                        restrict_to,
                        peers,
                        &click,
                    )
                {
                    // Switched to a single counter on another hex — never a
                    // member of this tile: drop the tile's markers.
                    for source in &sel.sources {
                        picker_ctx.commands.entity(*source).remove::<Selected>();
                    }
                }
            }
        }
    }
    None
}

/// The single-counter selection shared by the Idle arm and the tile arm of
/// the click handler: pick the counter nearest the cursor in the hex (stacks
/// fan out, so the rendered position decides), respecting the faction and
/// §1.1 command-scope gates. Leaves everything untouched (returns `false`)
/// when the click misses every counter or hits one the player may not take —
/// so a caller can keep its current selection instead.
///
/// Idle, a left-press on a placed unit selects it this way (single counter,
/// every phase). The whole-tile selection is the *double-click's* job in
/// every phase — group move in Movement ([`handle_stack_double_click`]),
/// fire group / melee attackers in the combat phases
/// ([`select_combat_tile`]) — so a click on a unit and a double-click
/// anywhere on its hex are cleanly distinguished.
///
/// `restrict_to`, when `Some`, is the only faction whose units may be picked
/// up -- set in bound multiplayer so a player can't grab an enemy counter on
/// their own turn.  `None` (unbound session / single-seat) allows selecting
/// either side, so solo play/testing can drive both factions. `peers`
/// additionally gates on the §1.1 multi-player command scopes.
fn select_single_unit(
    state: &mut PickerState,
    commands: &mut Commands,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    coord: HexCoord,
    game_state: &crate::GameStateResource,
    restrict_to: Option<omdurman_types::Player>,
    peers: &crate::peers::Peers,
    click: &ClickGeometry,
) -> bool {
    let Some(entity) = nearest_placed_unit_at(
        placed_units,
        coord,
        click.center,
        click.hit,
        click.stack_spread,
        click.hex_size,
    ) else {
        return false;
    };
    let Ok((_, placed)) = placed_units.get(entity) else {
        return false;
    };
    // §5.21: a "Friendlies" unit aboard a gunboat moves only with her, and
    // the two counters share the hex -- in the Movement phase a click that
    // lands on the passenger takes the gunboat (the disembarkation is
    // offered on its own card, whichever is selected).
    let (entity, placed) = matches!(game_state.0.phase, omdurman_rules::Phase::Movement)
        .then_some(placed.unit_id)
        .flatten()
        .and_then(|uid| game_state.0.find_unit(uid)?.state.loaded_on)
        .and_then(|carrier| {
            placed_units
                .iter()
                .find(|(_, p)| p.unit_id == Some(carrier))
        })
        .unwrap_or((entity, placed));

    if let Some(faction) = restrict_to
        && omdurman_rules::unit_profiles::section_owner(placed.section_name) != Some(faction)
    {
        return false; // not your unit -- ignore the click
    }
    if let Some(identity) = omdurman_rules::unit_profiles::identity_for_counter(
        placed.section_name,
        placed.col,
        placed.row,
    ) && !peers.scope_allows(&identity)
    {
        return false; // another member's command (§1.1) -- ignore the click
    }
    // Remaining allowance = full allowance minus what the unit has already
    // spent this turn (§5.11/§5.12), so re-selecting a unit that has partly
    // moved shows only its leftover movement -- not a fresh full budget. The
    // engine caps cumulatively regardless, but the overlay should reflect the
    // truth.
    let remaining_mp = unit_remaining_mp(game_state, placed);
    commands.entity(entity).try_insert(Selected);
    *state = PickerState::Selected {
        source: entity,
        start_coord: coord,
        remaining_mp,
        forced_stop: false,
    };
    true
}

/// The counters of `coord` a click may select alone, in a stable order:
/// those of `owner` this seat commands (§1.1).
fn stack_members(
    placed_units: &Query<(Entity, &PlacedUnit)>,
    coord: HexCoord,
    owner: Option<omdurman_types::Player>,
    peers: &crate::peers::Peers,
) -> Vec<Entity> {
    let mut members: Vec<Entity> = placed_units
        .iter()
        .filter(|(_, u)| u.coord == coord)
        .filter(|(_, u)| omdurman_rules::unit_profiles::section_owner(u.section_name) == owner)
        .filter(|(_, u)| {
            omdurman_rules::unit_profiles::identity_for_counter(u.section_name, u.col, u.row)
                .is_none_or(|identity| peers.scope_allows(&identity))
        })
        .map(|(e, _)| e)
        .collect();
    members.sort_by_key(|e| e.to_bits());
    members
}

/// The counter after `current` in `members`, round to the first.
fn next_member(members: &[Entity], current: Entity) -> Option<Entity> {
    let at = members.iter().position(|&e| e == current)?;
    members
        .get((at + 1) % members.len())
        .copied()
        .filter(|&e| e != current)
}

/// A press on the hex of the single selected counter `source`: select the
/// stack-mate under the cursor, or -- the selected counter clicked again --
/// the next counter of the stack, so every counter of a stack (a leader under
/// his battalions) is one or more single clicks away. `false` (nothing
/// changed) off the selection's hex, or with no other counter to take.
fn cycle_stack_selection(
    ctx: &mut PickerContext,
    source: Entity,
    coord: HexCoord,
    click: &ClickGeometry,
    game_state: &crate::GameStateResource,
    restrict_to: Option<omdurman_types::Player>,
    peers: &crate::peers::Peers,
) -> bool {
    let Ok((_, selected)) = ctx.placed_units.get(source) else {
        return false;
    };
    if selected.coord != coord {
        return false;
    }
    let owner = omdurman_rules::unit_profiles::section_owner(selected.section_name);
    if restrict_to.is_some_and(|f| owner != Some(f)) {
        return false;
    }
    let members = stack_members(&ctx.placed_units, coord, owner, peers);
    let under_cursor = nearest_placed_unit_at(
        &ctx.placed_units,
        coord,
        click.center,
        click.hit,
        click.stack_spread,
        click.hex_size,
    )
    .filter(|e| members.contains(e));
    let next = match under_cursor {
        Some(e) if e != source => Some(e),
        _ => next_member(&members, source),
    };
    let Some(next) = next else {
        return false;
    };
    let Ok((_, placed)) = ctx.placed_units.get(next) else {
        return false;
    };
    let remaining_mp = unit_remaining_mp(game_state, placed);
    ctx.commands.entity(source).remove::<Selected>();
    ctx.commands.entity(next).try_insert(Selected);
    *ctx.state = PickerState::Selected {
        source: next,
        start_coord: coord,
        remaining_mp,
        forced_stop: false,
    };
    true
}

/// Double-click stack selection: select *every* friendly unit on the hex for a
/// group move (movement phase). Units are kept in stable entity-id order and
/// each carries its own remaining-movement budget (`unit_remaining_mp`), which
/// is what makes slower units drop off along a shared path. Any stale
/// single-unit `Selected` marker outside the new stack is cleared so it can't
/// leak onto an unrelated counter, and a path plotted for the previous
/// selection is discarded: the new stack starts a path of its own (a leftover
/// leg would be submitted for the new stack's units and refused as a
/// non-contiguous route, §5.11).
fn handle_stack_double_click(
    state: &mut PickerState,
    commands: &mut Commands,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    coord: HexCoord,
    game_state: &crate::GameStateResource,
    restrict_to: Option<omdurman_types::Player>,
    peers: &crate::peers::Peers,
    movement_path: &mut MovementPath,
) {
    // The whole-stack selection covers only the *normal* stack: disrupted
    // counters cannot receive orders (they stay individually clickable for
    // inspection), so including them would just dead-weight the selection.
    let mut sources: Vec<Entity> = placed_units
        .iter()
        .filter(|(_, u)| u.coord == coord && !u.disrupted)
        .filter(|(_, u)| match restrict_to {
            Some(faction) => {
                omdurman_rules::unit_profiles::section_owner(u.section_name) == Some(faction)
            }
            None => true,
        })
        .filter(|(_, u)| {
            omdurman_rules::unit_profiles::identity_for_counter(u.section_name, u.col, u.row)
                .is_none_or(|identity| peers.scope_allows(&identity))
        })
        .map(|(e, _)| e)
        .collect();
    sources.sort_by_key(|e| e.to_bits());
    if sources.is_empty() {
        return;
    }
    movement_path.reset();
    // Say who of the hex stays: a disrupted counter cannot move (it was
    // left behind without a word, and the stack went on without it).
    let stay: Vec<String> = placed_units
        .iter()
        .filter(|(_, u)| u.coord == coord && u.disrupted)
        .filter(|(_, u)| match restrict_to {
            Some(faction) => {
                omdurman_rules::unit_profiles::section_owner(u.section_name) == Some(faction)
            }
            None => true,
        })
        .filter_map(|(_, u)| u.unit_id)
        .map(|id| crate::combat_ui::unit_name(id, &game_state.0))
        .collect();
    if !stay.is_empty() {
        movement_path.notice = Some(format!(
            "{} {} behind: disrupted units may not move until they recover at the end of \
             their side's turn.",
            crate::combat_ui::tally_names(stay.clone()).join(", "),
            if stay.len() == 1 {
                "is disrupted and stays"
            } else {
                "are disrupted and stay"
            },
        ));
    }
    // Clear stale markers from whichever single selection the first click of
    // the pair left behind (if it isn't part of the stack).
    match &*state {
        PickerState::Selected { source, .. } => {
            if !sources.contains(source) {
                commands.entity(*source).remove::<Selected>();
            }
        }
        PickerState::SelectedStack(old) => {
            for e in &old.sources {
                if !sources.contains(e) {
                    commands.entity(*e).remove::<Selected>();
                }
            }
        }
        PickerState::SelectedTile(old) => {
            for e in &old.sources {
                if !sources.contains(e) {
                    commands.entity(*e).remove::<Selected>();
                }
            }
        }
        _ => {}
    }
    let initial_mp: Vec<i16> = sources
        .iter()
        .map(|&e| {
            placed_units
                .get(e)
                .map(|(_, p)| unit_remaining_mp(game_state, p))
                .unwrap_or(0)
        })
        .collect();
    for &e in &sources {
        commands.entity(e).try_insert(Selected);
    }
    *state = PickerState::SelectedStack(StackSelection {
        sources,
        start_coord: coord,
        remaining_mp: initial_mp.clone(),
        initial_mp,
        forced_stop: false,
    });
}

/// Whole-tile combat selection — the unified fire/melee interaction model
/// (§6.14/§6.15/§7), invoked by a *double-click anywhere on the hex* (the
/// combat-phase counterpart of the movement group move; detection is
/// hex-based, so neither click of the pair must land on a counter symbol).
/// It selects every counter on the hex that can act in the current phase:
///
/// * fire sub-phase: non-disrupted units *with a fire factor* (a disrupt or a
///   leader without a factor would otherwise poison the whole-tile attack);
/// * Melee: non-disrupted *melee-capable* units (`build_melee_attack` gathers
///   exactly the co-stacked attackers).
///
/// A single click still selects exactly the counter under the cursor
/// ([`select_single_unit`]) — single-unit fire (§6.13) and a lone attacker
/// stay directly issuable.
///
/// Faction (bound games) and §1.1 command-scope filters apply as everywhere
/// else. Any stale selection markers outside the new tile are cleared so they
/// can't leak onto an unrelated counter. Returns `true` when a selection was
/// made.
fn select_combat_tile(
    state: &mut PickerState,
    commands: &mut Commands,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    game_state: &crate::GameStateResource,
    coord: HexCoord,
    restrict_to: Option<omdurman_types::Player>,
    peers: &crate::peers::Peers,
) -> bool {
    let in_fire = matches!(
        game_state.0.phase,
        omdurman_rules::Phase::OffensiveFire(_) | omdurman_rules::Phase::DefensiveFire(_)
    );
    let mut sources: Vec<Entity> = placed_units
        .iter()
        .filter(|(_, u)| u.coord == coord && !u.disrupted)
        .filter(|(_, u)| match restrict_to {
            Some(faction) => {
                omdurman_rules::unit_profiles::section_owner(u.section_name) == Some(faction)
            }
            None => true,
        })
        .filter(|(_, u)| {
            omdurman_rules::unit_profiles::identity_for_counter(u.section_name, u.col, u.row)
                .is_none_or(|identity| peers.scope_allows(&identity))
        })
        .filter(|(_, u)| {
            // Only counters that can act in this phase join the tile: firing
            // units in a fire sub-phase, melee-capable units in Melee.
            u.unit_id.is_some_and(|uid| {
                game_state.0.find_unit(uid).is_some_and(|unit| {
                    if in_fire {
                        unit.profile.fire.is_some()
                    } else {
                        unit.profile.kind.may_melee_attack()
                    }
                })
            })
        })
        .map(|(e, _)| e)
        .collect();
    sources.sort_by_key(|e| e.to_bits());
    if sources.is_empty() {
        return false;
    }
    // Clear stale markers from whichever previous selection is still around
    // (it isn't part of the new tile).
    let old_sources: Vec<Entity> = match &*state {
        PickerState::Selected { source, .. } => vec![*source],
        PickerState::SelectedStack(old) => old.sources.clone(),
        PickerState::SelectedTile(old) => old.sources.clone(),
        _ => Vec::new(),
    };
    for e in old_sources {
        if !sources.contains(&e) {
            commands.entity(e).remove::<Selected>();
        }
    }
    for &e in &sources {
        commands.entity(e).try_insert(Selected);
    }
    *state = PickerState::SelectedTile(TileSelection {
        sources,
        start_coord: coord,
    });
    true
}

/// Clear every selection marker and drop the selection — used by the
/// phase-change watcher so a selection plotted in one phase can never act in
/// the next.
fn clear_selection(state: &mut PickerState, commands: &mut Commands) {
    let sources: Vec<Entity> = match &*state {
        PickerState::Selected { source, .. } => vec![*source],
        PickerState::SelectedStack(sel) => sel.sources.clone(),
        PickerState::SelectedTile(sel) => sel.sources.clone(),
        _ => return,
    };
    for e in sources {
        commands.entity(e).remove::<Selected>();
    }
    *state = PickerState::Idle;
}

/// Reset the selection whenever the engine phase changes (a `Local` snapshot,
/// so the reset also fires on replay / snapshot convergence where no local
/// click precedes the advance). A selection is phase-shaped — a movement
/// group, a fire tile, a melee tile — so carrying it across a phase boundary
/// would let it act with stale semantics in the new phase.
pub(crate) fn reset_selection_on_phase_change(
    mut state: ResMut<PickerState>,
    mut commands: Commands,
    gs: Res<crate::GameStateResource>,
    mut last_phase: Local<Option<omdurman_rules::Phase>>,
) {
    if *last_phase != Some(gs.0.phase) {
        clear_selection(&mut state, &mut commands);
        *last_phase = Some(gs.0.phase);
    }
}

/// Build the rules [`UnitPlacement`] that deploying the picker unit at
/// `unit_idx` onto `coord` would produce. Used to gate both the placement
/// preview and the click on the *same* engine predicate
/// ([`GameState::can_deploy_unit`]: phase + zone + full stacking, §9.2/§9.3),
/// so the preview can never show green for a hex the click (or the apply path)
/// would reject. Returns `None` if the sprite has no `UnitId`/profile (never
/// for a visible picker unit).
pub(crate) fn deploy_candidate(
    picker: &UnitPicker,
    unit_idx: usize,
    coord: HexCoord,
) -> Option<UnitPlacement> {
    let unit = picker.available.get(unit_idx)?;
    let id = unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)?;
    let profile = omdurman_rules::unit_profiles::profile_for_unit(id)?;
    Some(UnitPlacement {
        id,
        position: coord,
        profile,
        state: UnitState::default(),
    })
}

/// Resolve a click while placing a counter.
fn place_click(
    ctx: &mut PickerContext,
    origin: Vec2,
    released: bool,
    unit_idx: usize,
    drag_drop: bool,
    coord: HexCoord,
    game_state: &crate::GameStateResource,
) -> Result<Option<GameEvent>, String> {
    if released && !drag_drop {
        *ctx.state = PickerState::Placing {
            unit_idx,
            preview_hex: None,
            preview_valid: false,
            drag_drop: false,
        };
        return Ok(None);
    }

    let Some(unit) = ctx.picker.available.get(unit_idx) else {
        *ctx.state = PickerState::Idle;
        return Ok(None);
    };

    // Gate placement on the *same* engine predicate the apply path uses
    // (phase + deployment zone + full stacking, §9.2/§9.3/§5.51-5.53), so
    // the preview ring and the click can never disagree with what the
    // engine will accept on the sequenced echo (barring a race between
    // peers). Outside Setup and Movement there is no engine predicate to
    // consult, so it falls back to passable-and-vacant.
    //
    // Movement-phase placement is *reinforcement entry* (§9.112/§9.113
    // Campaign order of appearance, §9.322 FoK entry edge): the click
    // will apply `PlaceReinforcements`, so the gate runs that check.
    // The engine's refusal, if any: reported to the player (the unit stays
    // in hand) instead of silently dropping the selection.
    let mut refusal = None;
    let candidate = deploy_candidate(&ctx.picker, unit_idx, coord);
    let engine_check = match game_state.0.phase {
        omdurman_rules::Phase::Setup => Some(candidate.map(|c| game_state.0.can_deploy_unit(&c))),
        omdurman_rules::Phase::Movement => {
            Some(candidate.map(|c| game_state.0.can_place_single_reinforcement(&c)))
        }
        _ => None,
    };
    let can_place = if let Some(check) = engine_check {
        // No candidate (unresolvable sprite): not placeable, no reason.
        refusal = check.as_ref().and_then(|r| r.clone().err());
        check.is_some_and(|r| r.is_ok())
    } else {
        let occupied = ctx.placed_units.iter().any(|(_, u)| u.coord == coord);
        !occupied && coord_passable(game_state.0.board, coord, unit.is_boat)
    };

    if can_place {
        let pos = hex_world_pos(coord, origin, &ctx.overlay.params);
        let unit = ctx.picker.available.remove(unit_idx);

        // Boat-ness from the sprite profile (the engine's source of truth),
        // not the cached `PickerUnit.is_boat` flag -- that flag is only
        // populated lazily from annotations and is unreliable (e.g. it
        // resets to `false` when a counter is returned to the picker). This
        // keeps the spawned counter and the wire event consistent with what
        // `can_deploy_unit`/`apply_effect` decided.
        let is_boat = unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
            .and_then(omdurman_rules::unit_profiles::profile_for_unit)
            .is_some_and(|p| p.kind.is_boat());

        // Optimistic sprite: shown at once, bound to its rules unit (or
        // dropped, if the engine rejects the placement) by
        // `reconcile_unit_sprites` when the sequenced echo is applied.
        let optimistic = spawn_placed_unit(
            &mut ctx.commands,
            &mut ctx.meshes,
            &mut ctx.materials,
            unit.handle.clone(),
            &ctx.overlay,
            pos,
            PlacedUnit {
                coord,
                section_name: unit.section_name,
                col: unit.col,
                row: unit.row,
                is_boat,
                unit_id: None,
                disrupted: false,
            },
        );
        ctx.commands
            .entity(optimistic)
            .try_insert(PendingPlacement::default());

        info!(
            section_name = %unit.section_name,
            col = unit.col,
            row = unit.row,
            coord.q = coord.q,
            coord.r = coord.r,
            "placing unit"
        );
        // Auto-select the next available unit in the same section so the
        // player can keep placing without returning to the picker panel.
        if ctx.picker.auto_place_next {
            // After `remove(unit_idx)` the next unit is now at the same
            // index (or we've reached the end).  Scan forward for the
            // next visible unit in the same section.
            // The next counter of the same tray group (see
            // `sidebar::tray_group`), as the tray shows them.
            let group_of = |u: &PickerUnit| {
                unit_id_for_section_pos(u.section_name, u.col as u8, u.row as u8)
                    .and_then(omdurman_rules::unit_profiles::profile_for_unit)
                    .map(|p| super::sidebar::tray_group(&p.identity))
            };
            let group = group_of(&unit);
            let next = ctx
                .picker
                .available
                .iter()
                .skip(unit_idx)
                .position(|u| u.shown() && group.is_some() && group_of(u) == group)
                .map(|p| unit_idx + p);
            if let Some(next_idx) = next {
                *ctx.state = PickerState::Placing {
                    unit_idx: next_idx,
                    preview_hex: None,
                    preview_valid: false,
                    drag_drop: false,
                };
            } else {
                *ctx.state = PickerState::Idle;
            }
        } else {
            *ctx.state = PickerState::Idle;
        }
        return Ok(Some(GameEvent::PlaceUnit {
            sprite: omdurman_types::SpriteRef {
                section_name: unit.section_name,
                col: unit.col,
                row: unit.row,
            },
            coord: omdurman_types::HexCoord::new(coord.q, coord.r),
            is_boat,
        }));
    }
    if let Some(error) = refusal {
        // Keep the counter in hand so the player can try another hex.
        return Err(error.to_string());
    }
    *ctx.state = PickerState::Idle;
    Ok(None)
}

/// The slip for a destination the route finder cannot reach (§5.11-§5.4:
/// movement points, terrain, zones of control, blocked hexsides): what the
/// cheapest route would cost, when there is one at any price, with the rule
/// that usually explains it for this kind of mover.
fn no_route_reason(goal: HexCoord, budget: i16, cheapest: Option<i16>, why: RouteWhy) -> String {
    let RouteWhy { boat, night } = why;
    let mut why = if boat {
        "going upstream a gunboat has its smaller upstream allowance, §5.24; it stops on \
         entering an enemy gunboat's zone of control, §5.41, §5.43"
            .to_string()
    } else {
        "Rough and Swamp cost 3 MP a hex, a khor +5, and no route passes through an \
         enemy zone of control: §5.11, §5.43"
            .to_string()
    };
    if night {
        why.push_str("; at night Anglo-Egyptian movement allowances are halved, §8.1");
    }
    match cheapest {
        Some(cost) => format!(
            "{goal} is out of reach: the cheapest route costs {cost} MP, this move has \
             {budget} MP left ({why}). The outlined hexes are in reach."
        ),
        None => format!(
            "No route to {goal}: zones of control (§5.4), enemy units or impassable \
             terrain and hexsides block every way there."
        ),
    }
}

/// What a refusal's explanation depends on besides the numbers.
#[derive(Clone, Copy, Default)]
struct RouteWhy {
    /// A gunboat (§5.24 allowances, gunboat ZOC) rather than a land unit.
    boat: bool,
    /// A night turn halving this mover's allowance (§8.1).
    night: bool,
}

/// Price a goal the route finder could not reach within the move's budget:
/// the cheapest route's cost at any price (`None`: no route at all), and the
/// MP this move has for that route -- a gunboat's upstream allowance once
/// the route runs upstream (§5.24), else `budget`.
fn price_unreachable(
    placed_units: &Query<(Entity, &PlacedUnit)>,
    placed: &PlacedUnit,
    (start, goal): (HexCoord, HexCoord),
    budget: i16,
    gunboat: Option<GunboatBudget>,
    game_state: &crate::GameStateResource,
) -> (Option<i16>, i16) {
    let Some((route, cost)) = cheapest_route(
        placed_units,
        placed,
        start,
        goal,
        i16::MAX,
        None,
        game_state,
    ) else {
        return (None, budget);
    };
    let have = match gunboat {
        Some(gunboat) => {
            let mut from = start;
            let upstream = route.iter().any(|&hex| {
                let up = step_is_upstream(&game_state.0, from, hex);
                from = hex;
                up
            });
            budget.min(gunboat.left(upstream))
        }
        None => budget,
    };
    (Some(cost), have)
}

/// Why no route reaches `goal`: when the goal itself may not hold one of the
/// `movers` (§5.51-§5.53 stacking: another tribe's stack, a leader outside
/// his command, a full hex) the route finder never even tries it -- say so,
/// rather than blaming terrain and zones of control.
fn unroutable_reason(
    gs: &omdurman_rules::effects::GameState,
    movers: &[omdurman_rules::UnitId],
    goal: HexCoord,
    budget: i16,
    cheapest: Option<i16>,
) -> String {
    // A unit that may not move at all this turn (a mine or chain stop, a
    // loading gunboat, a ZOC stop...) has no route anywhere: say which rule
    // holds it, not that its movement points ran out.
    let blocked = movers
        .iter()
        .find_map(|&id| gs.movement_blocker(id).map(|why| (id, why)));
    if let Some((id, why)) = blocked {
        let name = crate::combat_ui::unit_name(id, gs);
        return format!("{name} cannot move this turn: {why}.");
    }
    // The goal is no ground this unit can stand on: gunboats keep to the
    // Nile, land units off it (§5.22).
    let wrong_ground = movers.iter().find_map(|&id| {
        let mover = gs.find_unit(id)?;
        let is_boat = mover.profile.kind.is_boat();
        (!gs.on_deployable_terrain(goal, is_boat)).then_some(is_boat)
    });
    if let Some(is_boat) = wrong_ground {
        return if is_boat {
            format!("{goal} is not on the Nile: gunboats move only along the river (§5.22).")
        } else {
            format!("{goal} is the Nile: only gunboats move on the river (§5.22).")
        };
    }
    // A unit whose allowance this turn is spent: it already moved -- not "out
    // of reach … 0 MP left" (§5.11, §5.13).
    if budget <= 0
        && let Some(&id) = movers.iter().find(|&&id| gs.mp_spent(id) > 0)
    {
        let name = crate::combat_ui::unit_name(id, gs);
        return format!(
            "{name} has already moved this turn and has no movement points left (§5.11, \
             §5.13: unused points are lost, never carried over)."
        );
    }
    let stacking_refusal = movers.iter().find_map(|&id| {
        let mover = gs.find_unit(id)?;
        gs.check_stacking(mover, goal).err()
    });
    if let Some(error) = stacking_refusal {
        return format!("Cannot end a move on {goal}: {error}.");
    }
    let units: Vec<&omdurman_rules::UnitPlacement> =
        movers.iter().filter_map(|&id| gs.find_unit(id)).collect();
    // §5.23: only some units may enter the walled city of Omdurman -- a gate
    // or breach does not help the others, so do not send them looking.
    let barred = units.iter().find(|u| {
        gs.scenario != omdurman_types::Scenario::FallOfKhartoum
            && gs.board.is_walled_city(goal)
            && !gs.board.is_walled_city(u.position)
            && !u.profile.identity.may_enter_walled_city()
    });
    if let Some(unit) = barred {
        let name = crate::combat_ui::unit_name(unit.id, gs);
        let who = match unit.profile.identity.owner() {
            omdurman_types::Player::Dervish => {
                "of the Dervish only the Khalifa, the artillery and the Taiasha may enter it"
            }
            omdurman_types::Player::AngloEgyptian => {
                "gunboats and the \"Friendlies\" may not enter it"
            }
        };
        return format!("{name} may not enter the walled city of Omdurman: {who} (§5.23).");
    }
    // Nothing reaches the goal at any price, and it lies against a city
    // wall: the wall is what stands in the way.
    if cheapest.is_none()
        && goal
            .neighbors()
            .iter()
            .any(|&n| gs.hexside_effective_is(goal, n, |k| k == omdurman_types::HexsideKind::Wall))
    {
        return format!(
            "No route to {goal}: the city wall is in the way. Units cross it only at a \
             gate or a breach -- artillery can breach it (§6.63)."
        );
    }
    // A move into or out of the Zariba: it is entered and left only by its
    // two end hexsides on the Nile, at +2 MP (§9.233) -- the hedge and
    // trench in between are no way through.
    // (Only once it stands: the Campaign's Zariba is built in play, §5.3.)
    let zariba_stands = gs.board.zariba.iter().any(|&hex| {
        hex.neighbors()
            .into_iter()
            .any(|n| gs.hexside_effective_is(hex, n, omdurman_types::HexsideKind::is_zariba))
    });
    if let Some(start) = units.first().map(|u| u.position)
        && zariba_stands
        && gs.board.is_zariba(start) != gs.board.is_zariba(goal)
    {
        let cost = cheapest.map_or(String::new(), |c| {
            format!(" The cheapest route that way costs {c} MP; this move has {budget} MP left.")
        });
        return format!(
            "{goal} lies {} the Zariba: units enter and leave it only by its two end \
             hexsides on the Nile, at +2 MP (§9.233).{cost}",
            if gs.board.is_zariba(goal) {
                "inside"
            } else {
                "outside"
            },
        );
    }
    let why = RouteWhy {
        boat: units.iter().any(|u| u.profile.kind.is_boat()),
        night: gs.day_night == omdurman_types::DayNight::Night
            && units
                .iter()
                .any(|u| u.profile.identity.owner() == omdurman_types::Player::AngloEgyptian),
    };
    no_route_reason(goal, budget, cheapest, why)
}

/// A selected unit's budget while its route is plotted.
struct SingleRoute {
    remaining_mp: i16,
    forced_stop: bool,
}

/// Plot the [`auto_route`] from `start` to `goal` through the ordinary
/// per-leg click path, stopping at the first leg it refuses.
fn plot_single_route(
    ctx: &mut PickerContext,
    plan: &mut SingleRoute,
    source: Entity,
    start: HexCoord,
    goal: HexCoord,
    game_state: &crate::GameStateResource,
) -> Result<(), String> {
    let Ok((_, placed)) = ctx.placed_units.get(source) else {
        return Ok(());
    };
    let budget = if plan.forced_stop {
        0
    } else {
        plan.remaining_mp
    };
    let gunboat = GunboatBudget::of(game_state, placed, &ctx.movement_path);
    let Some(route) = auto_route(
        &ctx.placed_units,
        placed,
        start,
        goal,
        budget,
        gunboat,
        game_state,
    ) else {
        info!(?start, ?goal, budget, "no legal route to the clicked hex");
        let movers: Vec<_> = placed.unit_id.into_iter().collect();
        let (cheapest, have) = price_unreachable(
            &ctx.placed_units,
            placed,
            (start, goal),
            budget,
            gunboat,
            game_state,
        );
        return Err(unroutable_reason(
            &game_state.0,
            &movers,
            goal,
            have,
            cheapest,
        ));
    };
    let mut from = start;
    for hex in route {
        single_click(ctx, plan, true, source, from, hex, game_state)?;
        match *ctx.state {
            PickerState::Selected {
                start_coord,
                remaining_mp,
                forced_stop,
                ..
            } if start_coord == hex => {
                from = hex;
                plan.remaining_mp = remaining_mp;
                plan.forced_stop = forced_stop;
            }
            _ => break,
        }
    }
    // The engine's per-leg check refused a step the search accepted:
    // say where the route stops instead of leaving it short silently.
    if from != goal {
        return Err(format!(
            "The route to {goal} stops at {from}: the next step was refused."
        ));
    }
    Ok(())
}

fn single_click(
    ctx: &mut PickerContext,
    plan: &mut SingleRoute,
    released: bool,
    source: Entity,
    start_coord: HexCoord,
    coord: HexCoord,
    game_state: &crate::GameStateResource,
) -> Result<(), String> {
    if !released {
        return Ok(());
    }
    if coord == start_coord {
        return Ok(());
    }
    let Ok((_, placed)) = ctx.placed_units.get(source) else {
        *ctx.state = PickerState::Idle;
        return Ok(());
    };
    // During Setup, `Selected` is focus-only (the player hits Del to return
    // the counter to the picker). There's no movement during deployment, so
    // don't build path legs -- bail without changing state.
    if matches!(game_state.0.phase, omdurman_rules::Phase::Setup) {
        return Ok(());
    }
    // §9.345 (FALL OF KHARTOUM): a gunboat lying on one Nile mouth
    // crosses to the other off the board, as a move of its own -- the
    // engine takes it as the single step of a move, so it is only
    // offered before any other leg is plotted, and ends the move.
    if let Some(uid) = placed.unit_id
        && placed.is_boat
        && game_state.0.is_nile_mouth_crossing(start_coord, coord)
    {
        if !ctx.movement_path.legs.is_empty() {
            return Err(format!(
                "The off-board crossing to {coord} is a move of its own (§9.345): \
                 undo the plotted steps first."
            ));
        }
        let cost = omdurman_rules::effects::NILE_MOUTH_CROSSING_MP;
        game_state
            .0
            .can_move_gunboat(uid, coord, &[coord], MovementPoints::new(cost))
            .map_err(|error| format!("Cannot cross to {coord} (§9.345): {error}"))?;
        ctx.movement_path.legs.push((start_coord, coord));
        ctx.movement_path.cost_so_far += cost;
        *ctx.state = PickerState::Selected {
            source,
            start_coord: coord,
            remaining_mp: 0,
            forced_stop: true,
        };
        return Ok(());
    }
    // A distant hex -- or a neighbour behind a wall -- plot the cheapest
    // legal route to it, leg by leg (round through a gate).
    if !start_coord.neighbors().contains(&coord)
        || hexside_blocks_step(&game_state.0, start_coord, coord)
    {
        return plot_single_route(ctx, plan, source, start_coord, coord, game_state);
    }

    let leg = movement_leg_check(&ctx.placed_units, placed, start_coord, coord, game_state);
    let affordable = leg.cost > 0
        && plan.remaining_mp >= leg.cost
        && gunboat_cap_ok(
            game_state,
            placed,
            &ctx.movement_path,
            start_coord,
            coord,
            leg.cost,
        );

    if leg.accepted(affordable, plan.forced_stop) {
        let new_remaining = plan.remaining_mp - leg.cost;
        info!(
            "path leg accepted: {:?} -> {:?}, cost={}, remaining_mp={}, entering_zoc={}",
            start_coord, coord, leg.cost, new_remaining, leg.entering_enemy_zoc
        );
        // Accumulate the leg in the path resource.
        ctx.movement_path.legs.push((start_coord, coord));
        ctx.movement_path.cost_so_far += leg.cost;

        // Stay in Selected so the player can confirm or (if not pinned by
        // ZOC) keep adding legs. §5.43 forces a stop once a leg enters an
        // enemy ZOC; the flag is sticky for the rest of the selection.
        *ctx.state = PickerState::Selected {
            source,
            start_coord: coord,
            remaining_mp: new_remaining,
            forced_stop: plan.forced_stop || leg.entering_enemy_zoc,
        };
        // No GameEvent yet — committed on confirm.
        Ok(())
    } else {
        info!(
            source = source.to_bits(),
            adjacent = leg.adjacent,
            enemy_occupied = leg.enemy_occupied,
            passable = leg.passable,
            affordable,
            stacking_ok = leg.stacking_ok,
            forced_stop = plan.forced_stop,
            entering_zoc = leg.entering_enemy_zoc,
            cost = leg.cost,
            remaining_mp = plan.remaining_mp,
            "path leg rejected",
        );
        ctx.movement_path.refused = Some(coord);
        // Say why, where a rule explains it (the click was silent).
        if let Some(reason) = rejected_leg_reason(
            &game_state.0,
            placed.unit_id,
            &ctx.movement_path,
            (start_coord, coord),
            &leg,
            plan.forced_stop,
        ) {
            return Err(reason);
        }
        // A rejected click does not discard the work already plotted: the
        // selection and the accumulated legs stay, so the player can click
        // a legal continuation, Backspace-undo a leg, Enter-confirm, or
        // right-click/Delete to cancel. (Previously a single bad click —
        // e.g. an over-cap friendly pass-through hex per §5.51 — nuked the
        // whole path.)
        Ok(())
    }
}

/// Why the picker refused the step `from -> to` for `mover`, when a rule
/// says so plainly: a unit that never moves (a fort, §5.25) or may not move
/// now (disrupted, stopped in a ZOC, ...), one that already used its
/// allowance this turn, a land unit stepping into the Nile (§5.22), a route
/// already stopped by an enemy zone of control (§5.43). `None` leaves the
/// refusal to the flash on the hex.
fn rejected_leg_reason(
    gs: &omdurman_rules::effects::GameState,
    mover: Option<omdurman_rules::UnitId>,
    path: &MovementPath,
    (from, to): (HexCoord, HexCoord),
    leg: &MovementLegCheck,
    forced_stop: bool,
) -> Option<String> {
    let id = mover?;
    let unit = gs.find_unit(id)?;
    let name = crate::combat_ui::unit_name(id, gs);
    if path.legs.is_empty() {
        if matches!(
            unit.profile.movement,
            omdurman_rules::UnitMovement::Immobile
        ) {
            return Some(format!(
                "{name} never moves: a fort stays where it was placed (§5.25)."
            ));
        }
        if let Some(why) = gs.movement_blocker(id) {
            return Some(format!("{name} cannot move this turn: {why}."));
        }
        if gs.remaining_movement(id) <= 0 && gs.mp_spent(id) > 0 {
            return Some(format!(
                "{name} has already moved this turn and has no movement points left \
                 (§5.11, §5.13)."
            ));
        }
    }
    if forced_stop {
        return Some(format!(
            "The route stops at {from}: it entered an enemy zone of control, and a unit \
             goes no further that turn (§5.43). Backspace undoes the last step."
        ));
    }
    if !leg.passable && !unit.profile.kind.is_boat() && gs.board.is_nile(to) {
        return Some(format!(
            "{to} is the Nile: only gunboats move on the river (§5.22)."
        ));
    }
    None
}

/// Commit the accumulated multi-leg path as a single MoveUnit event.
///
/// Called when the player confirms (Enter / UI button). The full path
/// is sent in one event so the rules engine processes it atomically.
pub(crate) fn commit_single_path(
    ctx: &mut PickerContext,
    source: Entity,
    game_state: &crate::GameStateResource,
) -> Result<Option<GameEvent>, String> {
    if ctx.movement_path.legs.is_empty() {
        return Ok(None);
    }
    let Ok((_, placed)) = ctx.placed_units.get(source) else {
        *ctx.state = PickerState::Idle;
        ctx.movement_path.reset();
        return Ok(None);
    };

    // The final destination is the last leg's `to`.
    let final_dest = ctx.movement_path.legs.last().unwrap().1;
    let total_cost = ctx.movement_path.cost_so_far;

    // §5.51: the stacking cap applies where the mover *ends* its move.
    // Pass-through legs are plotted freely; block the commit if the final
    // hex would exceed the cap (mirroring the engine's apply-time check),
    // and keep the selection so the player can route on past the hex, undo
    // a leg, or cancel — instead of committing a move the engine would
    // reject on echo (animate then snap back).
    if let Some(uid) = placed.unit_id
        && let Some(mover) = game_state.0.find_unit(uid)
        && let Err(error) = game_state.0.check_stacking(mover, final_dest)
    {
        warn!(
            section_name = %placed.section_name,
            final_dest.q = final_dest.q,
            final_dest.r = final_dest.r,
            "commit rejected: final destination breaks the §5.51 stacking cap",
        );
        return Err(format!("Cannot end a move on {final_dest}: {error}"));
    }

    // No local animation: the counter glides along this route once the
    // engine accepts the move on the echo (`reconcile_unit_sprites`).
    let path: Vec<HexCoord> = ctx.movement_path.legs.iter().map(|&(_, to)| to).collect();

    info!(
        section_name = %placed.section_name,
        legs = path.len(),
        total_cost,
        "committing path"
    );

    ctx.movement_path.reset();
    *ctx.state = PickerState::Idle;

    Ok(Some(GameEvent::MoveUnit {
        sprite: omdurman_types::SpriteRef {
            section_name: placed.section_name,
            col: placed.col,
            row: placed.row,
        },
        to_q: final_dest.q,
        to_r: final_dest.r,
        path,
    }))
}

/// A selected stack's budgets while its route is plotted (movement-phase
/// double-click). Like [`SingleRoute`] but per unit: a leg is accepted if
/// *any* unit can afford it, and only the affordable units are charged.
/// Slower units stop (drop) at their last affordable hex;
/// [`commit_stack_path`] turns the one plotted path into one `MoveUnit` per
/// unit, each along the longest prefix of the path its budget covers -- so
/// after the move every unit is independent.
struct StackRoute {
    remaining_mp: Vec<i16>,
    initial_mp: Vec<i16>,
    forced_stop: bool,
}

/// Stack variant of [`plot_single_route`]: routed with the
/// fastest unit's budget; slower units drop off along it as usual.
fn plot_stack_route(
    ctx: &mut PickerContext,
    plan: &mut StackRoute,
    sources: &[Entity],
    start: HexCoord,
    goal: HexCoord,
    game_state: &crate::GameStateResource,
) -> Result<(), String> {
    let Some(Ok((_, placed))) = sources.first().map(|&s| ctx.placed_units.get(s)) else {
        return Ok(());
    };
    // The whole group must fit at its goal (§5.51-5.53): checking only
    // the representative unit let a four-stack be routed onto a hex
    // holding two, and the commit then moved two and silently left the
    // rest behind.
    let movers: Vec<_> = sources
        .iter()
        .filter_map(|&s| ctx.placed_units.get(s).ok().and_then(|(_, p)| p.unit_id))
        .collect();
    if let Err(error) = game_state.0.check_group_stacking(&movers, goal) {
        return Err(format!(
            "The stack cannot end its move on {goal} together: {error}. Move fewer \
             units, or pick another hex."
        ));
    }
    let budget = if plan.forced_stop {
        0
    } else {
        plan.remaining_mp.iter().copied().max().unwrap_or(0)
    };
    let gunboat = GunboatBudget::of(game_state, placed, &ctx.movement_path);
    let Some(route) = auto_route(
        &ctx.placed_units,
        placed,
        start,
        goal,
        budget,
        gunboat,
        game_state,
    ) else {
        info!(?start, ?goal, budget, "no legal route to the clicked hex");
        let movers: Vec<_> = sources
            .iter()
            .filter_map(|&s| ctx.placed_units.get(s).ok().and_then(|(_, p)| p.unit_id))
            .collect();
        let (cheapest, have) = price_unreachable(
            &ctx.placed_units,
            placed,
            (start, goal),
            budget,
            gunboat,
            game_state,
        );
        return Err(unroutable_reason(
            &game_state.0,
            &movers,
            goal,
            have,
            cheapest,
        ));
    };
    let mut from = start;
    for hex in route {
        stack_click(ctx, plan, true, sources, from, hex, game_state)?;
        match &*ctx.state {
            PickerState::SelectedStack(sel) if sel.start_coord == hex => {
                from = hex;
                plan.remaining_mp = sel.remaining_mp.clone();
                plan.forced_stop = sel.forced_stop;
            }
            _ => break,
        }
    }
    // The engine's per-leg check refused a step the search accepted:
    // say where the route stops instead of leaving it short silently.
    if from != goal {
        return Err(format!(
            "The route to {goal} stops at {from}: the next step was refused."
        ));
    }
    Ok(())
}

fn stack_click(
    ctx: &mut PickerContext,
    plan: &mut StackRoute,
    released: bool,
    sources: &[Entity],
    start_coord: HexCoord,
    coord: HexCoord,
    game_state: &crate::GameStateResource,
) -> Result<(), String> {
    if !released {
        return Ok(());
    }
    if coord == start_coord {
        return Ok(());
    }
    let Ok((_, placed)) = ctx.placed_units.get(sources[0]) else {
        *ctx.state = PickerState::Idle;
        return Ok(());
    };
    // No movement during Setup (the stack selection itself is movement
    // phase only, but a stale state could outlive a phase change).
    if matches!(game_state.0.phase, omdurman_rules::Phase::Setup) {
        return Ok(());
    }
    // A distant hex -- or a neighbour behind a wall -- plot the cheapest
    // legal route to it, leg by leg (round through a gate).
    if !start_coord.neighbors().contains(&coord)
        || hexside_blocks_step(&game_state.0, start_coord, coord)
    {
        return plot_stack_route(ctx, plan, sources, start_coord, coord, game_state);
    }

    // Stacking pre-check uses the first unit as the group's representative
    // (the engine re-validates each unit's move at commit, so this is a
    // courtesy gate for the common case -- moving a stack into a hex that
    // already pushes past the §5.51 cap).
    let leg = movement_leg_check(&ctx.placed_units, placed, start_coord, coord, game_state);
    // A leg is affordable if at least one unit's budget covers it. Units
    // that can't afford it keep their remaining (they are dropped here).
    let affordable = leg.cost > 0
        && plan.remaining_mp.iter().any(|&mp| mp >= leg.cost)
        && gunboat_cap_ok(
            game_state,
            placed,
            &ctx.movement_path,
            start_coord,
            coord,
            leg.cost,
        );

    if leg.accepted(affordable, plan.forced_stop) {
        // Who of the stack reaches the new end, and who stopped short of it
        // -- told before the order is confirmed: a stack order once split
        // silently, and leaders ran on alone to be overrun (§6.51).
        let costs: Vec<i16> = ctx
            .movement_path
            .legs
            .iter()
            .chain(std::iter::once(&(start_coord, coord)))
            .map(|&(from, to)| floor_movement_cost(&game_state.0, from, to, placed.is_boat))
            .collect();
        let units: Vec<Option<omdurman_rules::UnitId>> = sources
            .iter()
            .map(|&s| ctx.placed_units.get(s).ok().and_then(|(_, p)| p.unit_id))
            .collect();
        match stack_split(
            &game_state.0,
            &units,
            &plan.initial_mp,
            &costs,
            start_coord,
            coord,
        ) {
            StackSplit::Together => {}
            StackSplit::Notice(notice) => ctx.movement_path.notice = Some(notice),
            StackSplit::LeadersAlone(reason) => {
                ctx.movement_path.refused = Some(coord);
                return Err(reason);
            }
        }
        let new_remaining: Vec<i16> = plan
            .remaining_mp
            .iter()
            .map(|&mp| if mp >= leg.cost { mp - leg.cost } else { mp })
            .collect();
        info!(
            "stack path leg accepted: {:?} -> {:?}, cost={}, affordable_units={}, entering_zoc={}",
            start_coord,
            coord,
            leg.cost,
            new_remaining
                .iter()
                .zip(&plan.remaining_mp)
                .filter(|(nr, mp)| **nr < **mp)
                .count(),
            leg.entering_enemy_zoc,
        );
        ctx.movement_path.legs.push((start_coord, coord));
        ctx.movement_path.cost_so_far += leg.cost;
        *ctx.state = PickerState::SelectedStack(StackSelection {
            sources: sources.to_vec(),
            start_coord: coord,
            remaining_mp: new_remaining,
            initial_mp: plan.initial_mp.clone(),
            forced_stop: plan.forced_stop || leg.entering_enemy_zoc,
        });
        Ok(())
    } else {
        info!(
            adjacent = leg.adjacent,
            enemy_occupied = leg.enemy_occupied,
            passable = leg.passable,
            affordable,
            stacking_ok = leg.stacking_ok,
            forced_stop = plan.forced_stop,
            entering_zoc = leg.entering_enemy_zoc,
            cost = leg.cost,
            "stack path leg rejected",
        );
        ctx.movement_path.refused = Some(coord);
        if let Some(reason) = rejected_leg_reason(
            &game_state.0,
            placed.unit_id,
            &ctx.movement_path,
            (start_coord, coord),
            &leg,
            plan.forced_stop,
        ) {
            return Err(reason);
        }
        // Keep the group and the plotted path: a rejected click (an
        // over-cap friendly pass-through hex per §5.51, a non-adjacent or
        // out-of-budget hex, or a post-ZOC stop) must not discard the route
        // already plotted. The player can click a legal continuation,
        // Backspace-undo, Enter-confirm, or right-click/Delete to cancel.
        Ok(())
    }
}

/// How many of the path's legs (`costs`, in order) each budget covers --
/// the longest affordable prefix, as [`commit_stack_path`] moves each unit.
fn legs_reached(initial_mp: &[i16], costs: &[i16]) -> Vec<usize> {
    initial_mp
        .iter()
        .map(|&budget| {
            let mut spent = 0i16;
            costs
                .iter()
                .take_while(|&&cost| {
                    spent += cost;
                    spent <= budget
                })
                .count()
        })
        .collect()
}

/// What a stack's newest plotted leg (`from` -> `to`, the last of `costs`)
/// does to the group.
#[derive(Debug, PartialEq)]
enum StackSplit {
    /// Every unit still on the route goes on.
    Together,
    /// Some units stop at `from` while the rest go on: say so (§5.11).
    Notice(String),
    /// Only leaders would go on, leaving their troops behind: refused -- a
    /// leader alone is overrun by the first attack (§6.51). Moving him on
    /// alone stays one single-counter order away.
    LeadersAlone(String),
}

fn stack_split(
    gs: &omdurman_rules::effects::GameState,
    units: &[Option<omdurman_rules::UnitId>],
    initial_mp: &[i16],
    costs: &[i16],
    from: HexCoord,
    to: HexCoord,
) -> StackSplit {
    let n = costs.len();
    let reach = legs_reached(initial_mp, costs);
    let name = |i: usize| {
        units[i]
            .map(|id| crate::combat_ui::unit_name(id, gs))
            .unwrap_or_else(|| "a unit".into())
    };
    let is_leader = |i: usize| {
        units[i].and_then(|id| gs.find_unit(id)).is_some_and(|u| {
            matches!(
                u.profile.kind,
                omdurman_types::UnitKind::BritishLeader { .. }
                    | omdurman_types::UnitKind::DervishLeader { .. }
            )
        })
    };
    let goers: Vec<usize> = (0..units.len()).filter(|&i| reach[i] >= n).collect();
    // Units that came this far and stop here, at the leg's start.
    let stoppers: Vec<usize> = (0..units.len())
        .filter(|&i| n > 0 && reach[i] == n - 1)
        .collect();
    if stoppers.is_empty() || goers.is_empty() {
        return StackSplit::Together;
    }
    let list = |ids: &[usize]| {
        crate::combat_ui::tally_names(ids.iter().map(|&i| name(i)).collect()).join(", ")
    };
    let troops_left = (0..units.len()).any(|i| reach[i] < n && !is_leader(i));
    if goers.iter().all(|&i| is_leader(i)) && troops_left {
        return StackSplit::LeadersAlone(format!(
            "Only {} could go on to {to}: {} stop at {from}, and a leader alone is overrun \
             by the first attack (§6.51). Plot a shorter route for the stack, or select \
             the leader alone to send him on.",
            list(&goers),
            list(&stoppers),
        ));
    }
    StackSplit::Notice(format!(
        "{} stop at {from} (no movement left); {} go on to {to} -- each unit has its own \
         movement allowance (§5.11).",
        list(&stoppers),
        list(&goers),
    ))
}

/// Commit the plotted path as one `MoveUnit` per unit, each along the
/// longest prefix of the path its remaining budget covers.
///
/// Units whose budget can't reach even the first hex stay put (no event).
/// This is what "lower-movement units are dropped along the path": every
/// unit stops at its last affordable hex, and afterwards the units are
/// plain independent counters again.
pub(crate) fn commit_stack_path(
    ctx: &mut PickerContext,
    initial_mp: &[i16],
    sources: &[Entity],
    game_state: &crate::GameStateResource,
) -> Vec<GameEvent> {
    if ctx.movement_path.legs.is_empty() {
        return Vec::new();
    }
    let mut events = Vec::new();

    for (i, &source) in sources.iter().enumerate() {
        // The unit's budget for the whole plotted path is what it had when
        // the stack was selected: `remaining_mp` has already been charged
        // for the plotted legs.
        let remaining = initial_mp[i];
        let Ok((_, placed)) = ctx.placed_units.get(source) else {
            continue;
        };
        // Longest prefix whose cumulative terrain cost fits the budget.
        let mut cum = 0i16;
        let mut prefix: Vec<HexCoord> = Vec::new();
        for &(from, to) in &ctx.movement_path.legs {
            let leg_cost = floor_movement_cost(&game_state.0, from, to, placed.is_boat);
            if cum + leg_cost > remaining {
                break;
            }
            cum += leg_cost;
            prefix.push(to);
        }
        if prefix.is_empty() {
            // Ran out of movement before the first leg: stays in place.
            continue;
        }
        let to = *prefix.last().unwrap();
        // §5.51 end-of-move cap, per unit: a unit whose final stop would
        // break stacking is skipped (the engine would reject exactly that
        // unit's event on echo anyway; the player can route it on past the
        // hex or cancel). Only the *stop* is checked — intermediate
        // pass-through legs never bind.
        let stacking_ok = match placed.unit_id.and_then(|uid| game_state.0.find_unit(uid)) {
            Some(mover) => game_state.0.check_stacking(mover, to).is_ok(),
            None => true,
        };
        if !stacking_ok {
            warn!(
                section_name = %placed.section_name,
                to.q = to.q,
                to.r = to.r,
                "stack commit: unit's final stop breaks the §5.51 stacking cap -- skipped",
            );
            continue;
        }
        info!(
            section_name = %placed.section_name,
            legs = prefix.len(),
            cost = cum,
            to.q = to.q,
            to.r = to.r,
            "committing stack path for unit",
        );
        events.push(GameEvent::MoveUnit {
            sprite: omdurman_types::SpriteRef {
                section_name: placed.section_name,
                col: placed.col,
                row: placed.row,
            },
            to_q: to.q,
            to_r: to.r,
            path: prefix,
        });
    }

    // The group move is over: the stack is deselected and each unit is an
    // independent counter again. If nothing was committed (every unit was
    // budget-dropped or stacking-skipped), keep the plot so the player can
    // extend or cancel instead of silently losing it.
    if events.is_empty() && !ctx.movement_path.legs.is_empty() {
        warn!("stack commit produced no moves; keeping the plotted path");
        return events;
    }
    for &source in sources {
        ctx.commands.entity(source).remove::<Selected>();
    }
    ctx.movement_path.reset();
    *ctx.state = PickerState::Idle;
    events
}

/// Clear the accumulated movement path when the picker is idle.
/// Runs every frame before the movement overlay so stale path data
/// is never rendered.
pub(crate) fn clear_movement_path_when_idle(
    state: Res<PickerState>,
    mut movement_path: ResMut<MovementPath>,
) {
    if matches!(&*state, PickerState::Idle) && !movement_path.legs.is_empty() {
        crate::ui_trace::path_cleared("selection dropped", &crate::ui_trace::Stamp::NONE);
        movement_path.reset();
    }
}

/// Did any of this frame's [`PickerCommand`]s ask for `want`? Drains the
/// reader (each handler keeps its own cursor).
fn commanded(reader: &mut MessageReader<PickerCommand>, want: PickerCommand) -> bool {
    reader.read().filter(|cmd| **cmd == want).count() > 0
}

/// Confirm a pending movement path on [`PickerCommand::ConfirmMove`] (Enter
/// or the actions panel's "Confirm move" button): if a path is pending, fire
/// the commit.
pub(crate) fn confirm_movement_path(
    mut commands_in: MessageReader<PickerCommand>,
    mut picker_ctx: PickerContext,
    game_state: Res<crate::GameStateResource>,
    peers: crate::peers::Peers,
    mut dispatches: Option<ResMut<crate::dispatch::Dispatches>>,
) {
    if !commanded(&mut commands_in, PickerCommand::ConfirmMove) {
        return;
    }
    if picker_ctx.movement_path.legs.is_empty() {
        return;
    }
    // Must be the owning player's turn.
    if !peers.may_act_now(&game_state.0) {
        return;
    }
    crate::ui_trace::path_confirmed(
        picker_ctx.movement_path.legs.len(),
        picker_ctx.movement_path.cost_so_far,
        &crate::ui_trace::Stamp::of(&game_state),
    );
    match ActiveSelection::snapshot(&picker_ctx.state) {
        ActiveSelection::Single { source, .. } => {
            match commit_single_path(&mut picker_ctx, source, &game_state) {
                Ok(Some(event)) => {
                    picker_ctx
                        .action_writer
                        .write(events::LocalAction { event });
                }
                Ok(None) => {}
                // A refused commit keeps the path: say why, so the player
                // can route on, undo a leg, or cancel.
                Err(reason) => {
                    if let Some(dispatches) = dispatches.as_deref_mut() {
                        dispatches.refuse(crate::submit::REFUSED_HEADER, reason);
                    }
                }
            }
        }
        ActiveSelection::Stack(sel) => {
            for event in
                commit_stack_path(&mut picker_ctx, &sel.initial_mp, &sel.sources, &game_state)
            {
                picker_ctx
                    .action_writer
                    .write(events::LocalAction { event });
            }
        }
        ActiveSelection::Placing { .. } | ActiveSelection::Idle => {}
        // A combat tile selection commits nothing on Enter (fire clicks
        // allocate, melee clicks declare).
        ActiveSelection::Tile(_) => {}
    }
}

/// Undo the last leg of the pending movement path on
/// [`PickerCommand::UndoStep`] (Backspace or the "Undo step" button). Refunds the leg's movement points and steps the planned position
/// back one hex; the path can then be re-extended or committed. Only acts on
/// the owning player's turn, while a unit is selected with a pending path.
pub(crate) fn undo_movement_leg(
    mut commands_in: MessageReader<PickerCommand>,
    mut picker_ctx: PickerContext,
    game_state: Res<crate::GameStateResource>,
    peers: crate::peers::Peers,
) {
    if !commanded(&mut commands_in, PickerCommand::UndoStep) {
        return;
    }
    if picker_ctx.movement_path.legs.is_empty() {
        return;
    }
    // Only the owning player may act.
    if !peers.may_act_now(&game_state.0) {
        return;
    }
    crate::ui_trace::path_undone(
        picker_ctx.movement_path.legs.len() - 1,
        &crate::ui_trace::Stamp::of(&game_state),
    );
    // Pop the last leg and refund its cost. The stored `(from, to)` pair is
    // exactly the leg that was charged, so recomputing with it matches what
    // was paid (terrain + §9.233 surcharge; gunboats pay a flat 1 per Nile
    // hex).
    let (from, to) = picker_ctx
        .movement_path
        .legs
        .pop()
        .expect("checked non-empty above");
    let is_boat = match ActiveSelection::snapshot(&picker_ctx.state) {
        ActiveSelection::Single { source, .. } => picker_ctx
            .placed_units
            .get(source)
            .is_ok_and(|(_, p)| p.is_boat),
        ActiveSelection::Stack(sel) => sel
            .sources
            .first()
            .and_then(|&e| picker_ctx.placed_units.get(e).ok())
            .is_some_and(|(_, p)| p.is_boat),
        _ => false,
    };
    let cost = floor_movement_cost(&game_state.0, from, to, is_boat);
    picker_ctx.movement_path.cost_so_far -= cost;
    match ActiveSelection::snapshot(&picker_ctx.state) {
        // Single unit: step the planned position back to the leg's `from`,
        // refund the MP, and clear the sticky ZOC `forced_stop` -- the popped
        // leg was necessarily the one that set it, since a forced stop blocks
        // further legs.
        ActiveSelection::Single {
            source,
            remaining_mp,
            ..
        } => {
            *picker_ctx.state = PickerState::Selected {
                source,
                start_coord: from,
                remaining_mp: remaining_mp + cost,
                forced_stop: false,
            };
        }
        // Stack: refund the leg to exactly the units that were charged for it.
        // A unit was charged iff what it has *already paid* this move
        // (initial - remaining) covers the popped leg's cumulative cost; the
        // others dropped before this leg and get nothing back.
        ActiveSelection::Stack(sel) => {
            let mut remaining_mp = sel.remaining_mp.clone();
            let threshold = picker_ctx.movement_path.cost_so_far + cost;
            for (i, rem) in remaining_mp.iter_mut().enumerate() {
                if sel.initial_mp[i] - *rem >= threshold {
                    *rem += cost;
                }
            }
            *picker_ctx.state = PickerState::SelectedStack(StackSelection {
                sources: sel.sources.clone(),
                start_coord: from,
                remaining_mp,
                initial_mp: sel.initial_mp.clone(),
                forced_stop: false,
            });
        }
        _ => {
            // Shouldn't happen: the path is only non-empty while a unit/stack
            // is selected. Restore a consistent state regardless.
            *picker_ctx.state = PickerState::Idle;
        }
    }
}

/// Narrow the selection to one stack member on
/// [`PickerCommand::SelectMember`] (a click on its name in the "Selected
/// units" panel): the member becomes a single selection with its own
/// remaining movement, exactly as if its counter had been clicked.
pub(crate) fn select_stack_member(
    mut commands_in: MessageReader<PickerCommand>,
    mut picker_ctx: PickerContext,
    game_state: Res<crate::GameStateResource>,
) {
    let Some(member) = commands_in
        .read()
        .filter_map(|cmd| match cmd {
            PickerCommand::SelectMember(entity) => Some(*entity),
            _ => None,
        })
        .last()
    else {
        return;
    };
    let Ok((_, placed)) = picker_ctx.placed_units.get(member) else {
        return;
    };
    let coord = placed.coord;
    let remaining_mp = unit_remaining_mp(&game_state, placed);
    let previous: Vec<Entity> = picker_ctx.state.sources().to_vec();
    for entity in previous {
        if entity != member {
            picker_ctx.commands.entity(entity).remove::<Selected>();
        }
    }
    picker_ctx.commands.entity(member).try_insert(Selected);
    picker_ctx.movement_path.legs.clear();
    picker_ctx.movement_path.cost_so_far = 0;
    *picker_ctx.state = PickerState::Selected {
        source: member,
        start_coord: coord,
        remaining_mp,
        forced_stop: false,
    };
}

/// Return the focused unit to the picker on [`PickerCommand::ReturnToTray`]
/// (Del or the "Return to tray" button), but only during the placement phase ([`Phase::Setup`]) -- a unit placed in a
/// prior phase is not removable. Mirrors the engine's `RemoveDeployedUnit`
/// gate (§9.2/§9.3). The engine re-validates phase + ownership on apply.
pub(crate) fn delete_selected_unit(
    mut commands_in: MessageReader<PickerCommand>,
    mut picker_ctx: PickerContext,
    game_state: Res<crate::GameStateResource>,
    peers: crate::peers::Peers,
    mut submit: crate::submit::CheckedSubmit,
) {
    if !commanded(&mut commands_in, PickerCommand::ReturnToTray) {
        return;
    }
    // Stack selections never reach Delete (movement phase only, no pickup).
    let PickerState::Selected { source, .. } = &*picker_ctx.state else {
        return;
    };
    // Only during Setup (the placement phase). Units placed in a prior phase
    // (e.g. once play has begun) may not be removed.
    if !matches!(game_state.0.phase, omdurman_rules::Phase::Setup) {
        return;
    }
    // The side deploying now (set-up is sequential, §9.111/§9.211/§9.321).
    if !peers.may_act_now(&game_state.0) {
        return;
    }
    let Ok((_, placed)) = picker_ctx.placed_units.get(*source) else {
        return;
    };
    crate::ui_trace::button("remove unit (Del)");
    let submitted = submit.submit(
        &game_state.0,
        GameEvent::RemoveUnit {
            sprite: omdurman_types::SpriteRef {
                section_name: placed.section_name,
                col: placed.col,
                row: placed.row,
            },
        },
    );
    if !submitted {
        // Refused (slip shown); keep the unit focused.
        return;
    }
    // Deselect; the apply path despawns the entity and returns the counter to
    // the picker.
    picker_ctx.commands.entity(*source).remove::<Selected>();
    *picker_ctx.state = PickerState::Idle;
}

/// [`PickerCommand::Cancel`] (Esc, right-click on the board, or the
/// "Cancel" button): drop the plotted path, the selection or pending
/// placement, and close the fire-allocation tray (its allocations are kept).
pub fn cancel_placement(
    mut commands_in: MessageReader<PickerCommand>,
    mut state: ResMut<PickerState>,
    mut movement_path: ResMut<MovementPath>,
    allocation: Option<ResMut<crate::fire_allocation::FireAllocationState>>,
) {
    if !commanded(&mut commands_in, PickerCommand::Cancel) {
        return;
    }
    if !movement_path.legs.is_empty() {
        crate::ui_trace::path_cleared("cancel", &crate::ui_trace::Stamp::NONE);
    }
    movement_path.reset();
    *state = PickerState::Idle;
    if let Some(mut allocation) = allocation
        && allocation.panel_open
    {
        allocation.panel_open = false;
    }
}

/// Clear every unit's movement path when the active player changes -- i.e. at
/// the end of a player's turn -- so the arrows show the moves made *this* turn
/// and reset for the next. The turn lives in the rules engine
/// (`GameState.active_player`); we watch it via a `Local` snapshot rather than a
/// dedicated event, so the reset also fires correctly on replay and snapshot
/// convergence, where no local "end turn" click occurs.
pub fn clear_paths_on_turn_change(
    gs: Res<crate::GameStateResource>,
    mut paths: ResMut<UnitPaths>,
    mut movement_path: ResMut<MovementPath>,
    mut last_active: Local<Option<omdurman_types::Player>>,
) {
    let stamp = crate::ui_trace::Stamp::of(&gs);
    let active = gs.0.active_player;
    if *last_active != Some(active) {
        if last_active.is_some() {
            crate::ui_trace::path_cleared("turn change", &stamp);
            paths.0.clear();
            movement_path.reset();
        }
        *last_active = Some(active);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn baggara(world: &mut World, coord: HexCoord, col: u32) -> Entity {
        world
            .spawn(PlacedUnit {
                coord,
                section_name: SectionName::Baggara,
                col,
                row: 0,
                is_boat: false,
                unit_id: None,
                disrupted: false,
            })
            .id()
    }

    /// Play-test repro: a leg plotted for one stack must not ride along when
    /// another stack is double-clicked -- it used to, and the new stack's
    /// commit was refused as a non-contiguous path (§5.11).
    #[test]
    fn a_new_stack_selection_discards_the_previous_path() {
        let mut world = World::new();
        let first = HexCoord::new(11, 21);
        let second = HexCoord::new(12, 23);
        baggara(&mut world, first, 0);
        let other = baggara(&mut world, second, 1);
        world.insert_resource(PickerState::default());
        // An unbound session: every counter is in scope.
        world.init_resource::<crate::seats::Seats>();
        world.insert_resource(crate::seats::LocalPlayerKey(omdurman_net::PlayerKey(1)));
        world.init_resource::<crate::seats::SeatPresence>();
        world.insert_resource(crate::GameStateResource(
            omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign),
        ));
        world.insert_resource(MovementPath {
            legs: vec![(first, HexCoord::new(12, 21))],
            cost_so_far: 3,
            ..Default::default()
        });
        world
            .run_system_once(
                move |mut state: ResMut<PickerState>,
                      mut commands: Commands,
                      placed: Query<(Entity, &PlacedUnit)>,
                      peers: crate::peers::Peers,
                      game_state: Res<crate::GameStateResource>,
                      mut path: ResMut<MovementPath>| {
                    handle_stack_double_click(
                        &mut state,
                        &mut commands,
                        &placed,
                        second,
                        &game_state,
                        None,
                        &peers,
                        &mut path,
                    );
                },
            )
            .expect("the selection system runs");
        assert!(world.resource::<MovementPath>().legs.is_empty());
        assert!(matches!(
            world.resource::<PickerState>(),
            PickerState::SelectedStack(sel) if sel.sources == vec![other] && sel.start_coord == second
        ));
    }

    /// A gunboat stopped by a mine (§10.12) may not move on this turn: the
    /// slip names that, not the §5.24 upstream allowance.
    #[test]
    fn a_unit_that_may_not_move_is_told_why() {
        use omdurman_rules::{UnitId, UnitPlacement, unit_profiles::profile_for_unit};
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        gs.phase = omdurman_rules::Phase::Movement;
        gs.active_player = omdurman_types::Player::AngloEgyptian;
        let boat = UnitId::ALL
            .iter()
            .copied()
            .find(|id| {
                profile_for_unit(*id).is_some_and(|p| {
                    p.kind.is_boat() && p.identity.owner() == omdurman_types::Player::AngloEgyptian
                })
            })
            .expect("a British gunboat");
        gs.units.push(UnitPlacement {
            id: boat,
            position: HexCoord::new(34, 12),
            profile: profile_for_unit(boat).unwrap(),
            state: Default::default(),
        });
        gs.gunboats_stopped_this_turn.push(boat);
        let reason = unroutable_reason(&gs, &[boat], HexCoord::new(34, 9), 12, Some(3));
        assert!(reason.contains("cannot move this turn"), "{reason}");
        assert!(reason.contains("§10.12"), "{reason}");
        assert!(!reason.contains("§5.24"), "{reason}");
    }

    /// Play-test repro: Ali Wad Helu ordered onto a Jaalin stack was refused
    /// as "No route ... terrain costs, zones of control"; the real reason is
    /// §5.53 stacking at the goal.
    #[test]
    fn an_illegal_goal_stack_is_reported_as_stacking_not_routing() {
        use omdurman_rules::{UnitId, UnitPlacement, unit_profiles::profile_for_unit};
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        gs.phase = omdurman_rules::Phase::Movement;
        gs.active_player = omdurman_types::Player::Dervish;
        let goal = HexCoord::new(16, 30);
        for (id, position) in [
            (UnitId::AliWadHelu_0_0, HexCoord::new(15, 28)),
            (UnitId::JaalinI_0_1, goal),
        ] {
            gs.units.push(UnitPlacement {
                id,
                position,
                profile: profile_for_unit(id).expect("a counter"),
                state: Default::default(),
            });
        }
        let reason = unroutable_reason(&gs, &[UnitId::AliWadHelu_0_0], goal, 14, None);
        assert!(reason.contains("§5.53"), "{reason}");
        assert!(!reason.starts_with("No route"), "{reason}");
        // An empty goal keeps the routing explanation.
        let reason = unroutable_reason(
            &gs,
            &[UnitId::AliWadHelu_0_0],
            HexCoord::new(9, 9),
            14,
            None,
        );
        assert!(reason.starts_with("No route"), "{reason}");
        // A route that exists at some price says what it costs.
        let reason = unroutable_reason(
            &gs,
            &[UnitId::AliWadHelu_0_0],
            HexCoord::new(9, 9),
            14,
            Some(19),
        );
        assert!(reason.contains("costs 19 MP"), "{reason}");
    }

    /// Each unit of a stack covers the longest prefix of the route its own
    /// budget pays for (as the commit moves it).
    #[test]
    fn each_stack_member_reaches_what_its_budget_pays() {
        assert_eq!(legs_reached(&[15, 8, 3], &[3, 3, 3, 3]), vec![4, 2, 1]);
        assert_eq!(legs_reached(&[0], &[1]), vec![0]);
    }

    fn put(gs: &mut omdurman_rules::effects::GameState, id: UnitId, at: HexCoord) {
        gs.units.push(omdurman_rules::UnitPlacement {
            id,
            position: at,
            profile: omdurman_rules::unit_profiles::profile_for_unit(id).expect("a counter"),
            state: Default::default(),
        });
    }

    /// A stack order says who stops and who goes on; one that would send
    /// only the leader on, leaving his troops behind, is refused (§6.51).
    #[test]
    fn a_stack_split_is_told_and_a_lone_leader_is_held_back() {
        let mut gs = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        let leader = UnitId::ALL
            .iter()
            .copied()
            .find(|&id| {
                omdurman_rules::unit_profiles::profile_for_unit(id).is_some_and(|p| {
                    matches!(p.kind, omdurman_types::UnitKind::BritishLeader { .. })
                })
            })
            .expect("an Anglo-Egyptian leader");
        let battalion = UnitId::ALL
            .iter()
            .copied()
            .find(|&id| {
                omdurman_rules::unit_profiles::profile_for_unit(id).is_some_and(|p| {
                    p.identity.owner() == omdurman_types::Player::AngloEgyptian
                        && matches!(p.kind, omdurman_types::UnitKind::Infantry { .. })
                })
            })
            .expect("a battalion");
        let cavalry = UnitId::ALL
            .iter()
            .copied()
            .find(|&id| {
                omdurman_rules::unit_profiles::profile_for_unit(id).is_some_and(|p| {
                    p.identity.owner() == omdurman_types::Player::AngloEgyptian
                        && matches!(p.kind, omdurman_types::UnitKind::Cavalry { .. })
                })
            })
            .expect("a cavalry unit");
        let at = HexCoord::new(10, 10);
        for id in [leader, battalion, cavalry] {
            put(&mut gs, id, at);
        }
        let (from, to) = (HexCoord::new(11, 10), HexCoord::new(12, 10));
        // Everyone affords both legs: together.
        let units = [Some(leader), Some(battalion), Some(cavalry)];
        assert_eq!(
            stack_split(&gs, &units, &[15, 8, 12], &[3, 3], from, to),
            StackSplit::Together
        );
        // The battalion stops after the first leg, cavalry and leader go on.
        let StackSplit::Notice(notice) = stack_split(&gs, &units, &[15, 4, 12], &[3, 3], from, to)
        else {
            panic!("a split is told");
        };
        assert!(
            notice.contains("stop at (11, 10)") || notice.contains("stop at"),
            "{notice}"
        );
        // Only the leader could go on: refused.
        assert!(matches!(
            stack_split(&gs, &units, &[15, 4, 4], &[3, 3], from, to),
            StackSplit::LeadersAlone(_)
        ));
        // A leader alone with no troops left behind is no split at all.
        assert_eq!(
            stack_split(&gs, &[Some(leader)], &[15], &[3, 3], from, to),
            StackSplit::Together
        );
    }

    /// Clicking the selected counter again cycles round its stack.
    #[test]
    fn the_next_member_wraps_round_the_stack() {
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let c = world.spawn_empty().id();
        let members = [a, b, c];
        assert_eq!(next_member(&members, a), Some(b));
        assert_eq!(next_member(&members, c), Some(a));
        assert_eq!(next_member(&[a], a), None);
    }

    /// The reasons a refused move gives name the rule that bars it: a
    /// unit that already moved, the §5.23 walled-city eligibility.
    #[test]
    fn refusals_name_the_rule_that_bars_the_move() {
        let mut gs = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        gs.board = omdurman_rules::board_data::board_info(omdurman_types::MapKind::Campaign);
        gs.phase = omdurman_rules::Phase::Movement;
        gs.active_player = omdurman_types::Player::Dervish;
        let baggara = UnitId::Baggara_0_0;
        put(&mut gs, baggara, HexCoord::new(9, 9));
        gs.mp_spent_this_turn.insert(baggara, 9);
        let reason = unroutable_reason(&gs, &[baggara], HexCoord::new(12, 12), 0, Some(3));
        assert!(reason.contains("already moved"), "{reason}");
        // A Baggara may not enter the walled city (§5.23): no "breach it".
        gs.mp_spent_this_turn.clear();
        let city = *gs
            .board
            .walled_city
            .iter()
            .find(|h| gs.board.terrain_at(**h).is_some())
            .expect("Omdurman's walled city");
        let reason = unroutable_reason(&gs, &[baggara], city, 9, None);
        assert!(reason.contains("§5.23"), "{reason}");
        assert!(!reason.contains("breach"), "{reason}");
    }

    /// After the §9.345 crossing (six upstream MP) a gunboat has what its
    /// upstream allowance leaves, not its downstream one: the path panel
    /// read "10 left after" for a boat with 4.
    #[test]
    fn the_crossing_leaves_the_upstream_remainder() {
        let mut gs =
            omdurman_rules::effects::GameState::new(omdurman_types::Scenario::FallOfKhartoum);
        gs.board = omdurman_rules::board_data::board_info(omdurman_types::MapKind::FallOfKhartoum);
        gs.phase = omdurman_rules::Phase::Movement;
        gs.active_player = omdurman_types::Player::AngloEgyptian;
        gs.day_night = omdurman_types::DayNight::Day;
        let white = gs
            .board
            .hex_of_location(omdurman_types::Location::WhiteNileMouth)
            .expect("the White Nile mouth");
        let blue = *gs
            .nile_mouth_crossings(white)
            .first()
            .expect("a Blue Nile mouth");
        let boat = UnitId::ALL
            .iter()
            .copied()
            .find(|&id| {
                omdurman_rules::unit_profiles::profile_for_unit(id).is_some_and(|p| {
                    p.kind.is_boat() && p.identity.owner() == omdurman_types::Player::AngloEgyptian
                })
            })
            .expect("a gunboat");
        put(&mut gs, boat, white);
        let unit = gs.find_unit(boat).unwrap();
        let up = gs.gunboat_allowances(unit).unwrap().upstream.value() as i16;
        let path = MovementPath {
            legs: vec![(white, blue)],
            cost_so_far: omdurman_rules::effects::NILE_MOUTH_CROSSING_MP,
            ..Default::default()
        };
        assert_eq!(
            crate::picker::left_after_path(&gs, boat, &path),
            up - omdurman_rules::effects::NILE_MOUTH_CROSSING_MP
        );
    }
}

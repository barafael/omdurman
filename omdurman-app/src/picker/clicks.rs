//! Board click handling: the place / select / move state machine.

use super::*;
use crate::board_click::PickerClick;
use crate::hotkeys::PickerCommand;
use bevy::ecs::message::MessageReader;

/// Max seconds between two left-clicks on the same hex for them to count as a
/// double-click (select-the-whole-stack, movement phase).
const DOUBLE_CLICK_SECS: f64 = 0.35;

/// Bundle of `&mut PickerState` + `&mut Commands` for [`handle_idle_click`].
/// Plain struct (the consumer is not a system).
struct IdleSelectionCtx<'a, 'b, 'c> {
    state: &'a mut PickerState,
    commands: &'a mut Commands<'b, 'c>,
}

/// The spatial facts of one board click: the ground-plane hit point, the
/// clicked hex's centre in world space, and the stack spread used to pick
/// among stacked counters. Bundled so [`handle_idle_click`] stays under
/// clippy's argument limit.
struct ClickGeometry {
    hit: Vec3,
    center: Vec3,
    stack_spread: f32,
    hex_size: f32,
}

/// Bundles the picker-specific resources, queries, and command buffers so
/// [`handle_picker_clicks`] and [`unit_picker_ui`] stay under the
/// system-parameter limit.  Resources that are also consumed by *other* systems
/// (e.g. `HexLayout`, `GameMap`) are included here because they are logically
/// part of the picker's map-interaction domain; those systems continue to take
/// the individual `Res`s.
#[derive(bevy::ecs::system::SystemParam)]
pub struct PickerContext<'w, 's> {
    pub picker: ResMut<'w, UnitPicker>,
    pub state: ResMut<'w, PickerState>,
    pub movement_path: ResMut<'w, MovementPath>,
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
    pub game_map: Res<'w, GameMap>,
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
    game_state: Option<Res<crate::GameStateResource>>,
    peers: crate::peers::Peers,
    time: Res<Time>,
    // Carries the engine's reason when a placement click is refused.
    mut dispatches: Option<ResMut<crate::dispatch::Dispatches>>,
    mut last_click: Local<Option<(f64, HexCoord)>>,
) {
    let game_state = game_state.as_deref();
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
        if let (Some(reason), Some(dispatches)) = (refusal, dispatches.as_deref_mut()) {
            dispatches.push(crate::submit::REFUSED_HEADER, reason);
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
    game_state: Option<&crate::GameStateResource>,
    peers: &crate::peers::Peers,
    now: f64,
    last_click: &mut Option<(f64, HexCoord)>,
) -> Option<String> {
    // §turn-order: a unit may only be moved on its owner's turn. When a game is
    // live, gate interactive movement on the local player being the rules
    // engine's active player (`handle_idle_click`/move path below). Placement
    // during set-up is not gated. With no game state (editor) there is no gate.
    // (The click router already drops clicks outside the seat's phase unless
    // a counter is in hand; this stays as the picker's own backstop.)
    let may_move = game_state.is_none_or(|gs| peers.may_act_now(&gs.0));

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
    let scope_ok = |identity: &omdurman_rules::UnitIdentity| peers.scope_allows(identity);

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
        && game_state.is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup))
    {
        // Pick the specific counter under the cursor (a stack fans out, so the
        // nearest-by-rendered-position is the right one, not just the first).
        if let Some((entity, placed)) = nearest_placed_unit_at(
            &picker_ctx.placed_units,
            coord,
            center,
            hit,
            stack_spread,
            picker_ctx.overlay.params.hex_size,
        ) {
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
            ) && !scope_ok(&identity)
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
        match game_state.map(|gs| gs.0.phase) {
            Some(
                omdurman_rules::Phase::OffensiveFire(_)
                | omdurman_rules::Phase::DefensiveFire(_)
                | omdurman_rules::Phase::Melee,
            ) => {
                select_combat_tile(
                    &mut picker_ctx.state,
                    &mut picker_ctx.commands,
                    &picker_ctx.placed_units,
                    game_state,
                    coord,
                    restrict_to,
                    &scope_ok,
                );
            }
            // Movement and Setup keep the group-move stack selection (a
            // double-click elsewhere is a no-op).
            Some(omdurman_rules::Phase::Movement) | Some(omdurman_rules::Phase::Setup) | None => {
                handle_stack_double_click(
                    &mut picker_ctx.state,
                    &mut picker_ctx.commands,
                    &picker_ctx.placed_units,
                    coord,
                    game_state,
                    restrict_to,
                    &scope_ok,
                    &mut picker_ctx.movement_path,
                );
            }
        }
        return None;
    }

    match ActiveSelection::snapshot(&picker_ctx.state) {
        // Selecting a unit to move is only meaningful on your own turn.
        ActiveSelection::Idle if may_move => {
            handle_idle_click(
                pressed,
                coord,
                &picker_ctx.placed_units,
                IdleSelectionCtx {
                    state: &mut picker_ctx.state,
                    commands: &mut picker_ctx.commands,
                },
                game_state,
                restrict_to,
                &scope_ok,
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
                && game_state.is_some_and(|gs| {
                    !matches!(gs.0.phase, omdurman_rules::Phase::Setup) && !peers.may_act_now(&gs.0)
                }) =>
        {
            return Some(if peers.paused() {
                "The game is paused until the absent player returns.".to_string()
            } else {
                "Units may only enter on their own side's movement phase.".to_string()
            });
        }
        // (Deployment-zone and every other set-up refusal comes from the
        // engine through `PlacingClick::handle`, with its reason.)
        ActiveSelection::Placing {
            unit_idx,
            drag_drop,
        } => {
            let mut placing = PlacingClick {
                picker: &mut picker_ctx.picker,
                state: &mut picker_ctx.state,
                overlay: &picker_ctx.overlay,
                game_map: &picker_ctx.game_map,
                commands: &mut picker_ctx.commands,
                meshes: &mut picker_ctx.meshes,
                materials: &mut picker_ctx.materials,
                origin,
            };
            match placing.handle(
                &picker_ctx.placed_units,
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
        } if game_state.is_none_or(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Movement)) => {
            let mut sel = SelectedClick {
                state: &mut picker_ctx.state,
                game_map: &picker_ctx.game_map,
                remaining_mp,
                forced_stop,
                movement_path: &mut picker_ctx.movement_path,
            };
            // Legs only extend the path (committed on confirm); a refusal
            // is a destination out of reach.
            let refusal = sel
                .handle(
                    &picker_ctx.placed_units,
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
        // the next firer without cancelling first; a hex holding enemy
        // counters stays target territory for the release.
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
                let own_hex = picker_ctx
                    .placed_units
                    .get(source)
                    .is_ok_and(|(_, p)| p.coord == coord);
                if !holds_foreign
                    && !own_hex
                    && select_single_unit(
                        &mut picker_ctx.state,
                        &mut picker_ctx.commands,
                        &picker_ctx.placed_units,
                        coord,
                        game_state,
                        restrict_to,
                        &scope_ok,
                        &click,
                    )
                {
                    picker_ctx.commands.entity(source).remove::<Selected>();
                }
            }
        }
        ActiveSelection::Stack(sel)
            if game_state
                .is_none_or(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Movement)) =>
        {
            // Group move: the whole stack follows one plotted path. Legs charge
            // only the units that can afford them; slower units are dropped at
            // their last affordable hex. Commit (Enter) splits the path into
            // per-unit prefix `MoveUnit` events.
            let mut sel_click = SelectedStackClick {
                state: &mut picker_ctx.state,
                game_map: &picker_ctx.game_map,
                commands: &mut picker_ctx.commands,
                remaining_mp: sel.remaining_mp.clone(),
                initial_mp: sel.initial_mp.clone(),
                forced_stop: sel.forced_stop,
                movement_path: &mut picker_ctx.movement_path,
            };
            if let Err(reason) = sel_click.handle(
                &picker_ctx.placed_units,
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
                        &scope_ok,
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

/// Idle: a left-press on a placed unit selects it (single counter, every
/// phase).  During setup, it removes the unit from the board (re-pickup for
/// re-placement). The whole-tile selection is the *double-click's* job in
/// every phase — group move in Movement ([`handle_stack_double_click`]),
/// fire group / melee attackers in the combat phases
/// ([`select_combat_tile`]) — so a click on a unit and a double-click
/// anywhere on its hex are cleanly distinguished.
///
/// `restrict_to`, when `Some`, is the only faction whose units may be picked
/// up -- set in bound multiplayer so a player can't grab an enemy counter on
/// their own turn.  `None` (unbound session / single-seat) allows selecting
/// either side, so solo play/testing can drive both factions. `scope_ok`
/// additionally gates on the §1.1 multi-player command scopes.
#[allow(clippy::too_many_arguments)]
fn handle_idle_click(
    pressed: bool,
    coord: HexCoord,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    selection: IdleSelectionCtx,
    game_state: Option<&crate::GameStateResource>,
    restrict_to: Option<omdurman_types::Player>,
    scope_ok: &dyn Fn(&omdurman_rules::UnitIdentity) -> bool,
    click: &ClickGeometry,
) {
    let IdleSelectionCtx { state, commands } = selection;
    if !pressed {
        return;
    }
    select_single_unit(
        state,
        commands,
        placed_units,
        coord,
        game_state,
        restrict_to,
        scope_ok,
        click,
    );
}

/// The single-counter selection shared by the Idle arm and the tile arm of
/// the click handler: pick the counter nearest the cursor in the hex (stacks
/// fan out, so the rendered position decides), respecting the faction and
/// §1.1 command-scope gates. Leaves everything untouched (returns `false`)
/// when the click misses every counter or hits one the player may not take —
/// so a caller can keep its current selection instead.
#[allow(clippy::too_many_arguments)]
fn select_single_unit(
    state: &mut PickerState,
    commands: &mut Commands,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    coord: HexCoord,
    game_state: Option<&crate::GameStateResource>,
    restrict_to: Option<omdurman_types::Player>,
    scope_ok: &dyn Fn(&omdurman_rules::UnitIdentity) -> bool,
    click: &ClickGeometry,
) -> bool {
    let Some((entity, placed)) = nearest_placed_unit_at(
        placed_units,
        coord,
        click.center,
        click.hit,
        click.stack_spread,
        click.hex_size,
    ) else {
        return false;
    };

    if let Some(faction) = restrict_to
        && omdurman_rules::unit_profiles::section_owner(placed.section_name) != Some(faction)
    {
        return false; // not your unit -- ignore the click
    }
    if let Some(identity) = omdurman_rules::unit_profiles::identity_for_counter(
        placed.section_name,
        placed.col,
        placed.row,
    ) && !scope_ok(&identity)
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

/// Double-click stack selection: select *every* friendly unit on the hex for a
/// group move (movement phase). Units are kept in stable entity-id order and
/// each carries its own remaining-movement budget (`unit_remaining_mp`), which
/// is what makes slower units drop off along a shared path. Any stale
/// single-unit `Selected` marker outside the new stack is cleared so it can't
/// leak onto an unrelated counter, and a path plotted for the previous
/// selection is discarded: the new stack starts a path of its own (a leftover
/// leg would be submitted for the new stack's units and refused as a
/// non-contiguous route, §5.11).
#[allow(clippy::too_many_arguments)]
fn handle_stack_double_click(
    state: &mut PickerState,
    commands: &mut Commands,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    coord: HexCoord,
    game_state: Option<&crate::GameStateResource>,
    restrict_to: Option<omdurman_types::Player>,
    scope_ok: &dyn Fn(&omdurman_rules::UnitIdentity) -> bool,
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
                .is_none_or(|identity| scope_ok(&identity))
        })
        .map(|(e, _)| e)
        .collect();
    sources.sort_by_key(|e| e.to_bits());
    if sources.is_empty() {
        return;
    }
    movement_path.reset();
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
    game_state: Option<&crate::GameStateResource>,
    coord: HexCoord,
    restrict_to: Option<omdurman_types::Player>,
    scope_ok: &dyn Fn(&omdurman_rules::UnitIdentity) -> bool,
) -> bool {
    let in_fire = game_state.is_some_and(|gs| {
        matches!(
            gs.0.phase,
            omdurman_rules::Phase::OffensiveFire(_) | omdurman_rules::Phase::DefensiveFire(_)
        )
    });
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
                .is_none_or(|identity| scope_ok(&identity))
        })
        .filter(|(_, u)| {
            // Only counters that can act in this phase join the tile: firing
            // units in a fire sub-phase, melee-capable units in Melee.
            u.unit_id.is_some_and(|uid| {
                game_state
                    .and_then(|gs| gs.0.find_unit(uid))
                    .is_some_and(|unit| {
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
    game_state: Option<Res<crate::GameStateResource>>,
    mut last_phase: Local<Option<omdurman_rules::Phase>>,
) {
    let Some(gs) = game_state else { return };
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

/// Borrowed context for resolving a click while placing a counter.
///
/// The map query is *not* stored here -- it is passed to [`handle`](Self::handle)
/// as a parameter. `Query` is invariant over its data, so coupling its
/// world/state lifetimes to the struct's other borrows (notably `Commands`)
/// would make the struct unconstructible from a normal Bevy system.
struct PlacingClick<'a, 'w, 's> {
    picker: &'a mut UnitPicker,
    state: &'a mut PickerState,
    overlay: &'a HexOverlay,
    game_map: &'a GameMap,
    commands: &'a mut Commands<'w, 's>,
    meshes: &'a mut Assets<Mesh>,
    materials: &'a mut Assets<StandardMaterial>,
    origin: Vec2,
}

impl PlacingClick<'_, '_, '_> {
    fn handle(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        released: bool,
        unit_idx: usize,
        drag_drop: bool,
        coord: HexCoord,
        game_state: Option<&crate::GameStateResource>,
    ) -> Result<Option<GameEvent>, String> {
        if released && !drag_drop {
            *self.state = PickerState::Placing {
                unit_idx,
                preview_hex: None,
                preview_valid: false,
                drag_drop: false,
            };
            return Ok(None);
        }

        let Some(unit) = self.picker.available.get(unit_idx) else {
            *self.state = PickerState::Idle;
            return Ok(None);
        };

        // Gate placement on the *same* engine predicate the apply path uses
        // (phase + deployment zone + full stacking, §9.2/§9.3/§5.51-5.53), so
        // the preview ring and the click can never disagree with what the
        // engine will accept on the sequenced echo (barring a race between
        // peers). The editor / unbound non-setup path has no engine state to
        // consult, so it falls back to passable-and-vacant.
        //
        // Movement-phase placement is *reinforcement entry* (§9.112/§9.113
        // Campaign order of appearance, §9.322 FoK entry edge): the click
        // will apply `PlaceReinforcements`, so the gate runs that check.
        // The engine's refusal, if any: reported to the player (the unit stays
        // in hand) instead of silently dropping the selection.
        let mut refusal = None;
        let can_place = if let Some(gs) = game_state {
            let candidate = deploy_candidate(self.picker, unit_idx, coord);
            let engine_check = match gs.0.phase {
                omdurman_rules::Phase::Setup => Some(candidate.map(|c| gs.0.can_deploy_unit(&c))),
                omdurman_rules::Phase::Movement => {
                    Some(candidate.map(|c| gs.0.can_place_single_reinforcement(&c)))
                }
                _ => None,
            };
            if let Some(check) = engine_check {
                // No candidate (unresolvable sprite): not placeable, no reason.
                refusal = check.as_ref().and_then(|r| r.clone().err());
                check.is_some_and(|r| r.is_ok())
            } else {
                let occupied = placed_units.iter().any(|(_, u)| u.coord == coord);
                !occupied && coord_passable(self.game_map, coord, unit.is_boat)
            }
        } else {
            let occupied = placed_units.iter().any(|(_, u)| u.coord == coord);
            !occupied && coord_passable(self.game_map, coord, unit.is_boat)
        };

        if can_place {
            let pos = hex_world_pos(coord, self.origin, &self.overlay.params);
            let unit = self.picker.available.remove(unit_idx);

            // Boat-ness from the sprite profile (the engine's source of truth),
            // not the cached `PickerUnit.is_boat` flag -- that flag is only
            // populated lazily from annotations and is unreliable (e.g. it
            // resets to `false` when a counter is returned to the picker). This
            // keeps the spawned counter and the wire event consistent with what
            // `can_deploy_unit`/`apply_effect` decided.
            let is_boat =
                unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
                    .and_then(omdurman_rules::unit_profiles::profile_for_unit)
                    .is_some_and(|p| p.kind.is_boat());

            // Optimistic sprite: shown at once, bound to its rules unit (or
            // dropped, if the engine rejects the placement) by
            // `reconcile_unit_sprites` when the sequenced echo is applied.
            let optimistic = spawn_placed_unit(
                self.commands,
                self.meshes,
                self.materials,
                unit.handle.clone(),
                self.overlay,
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
            self.commands
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
            if self.picker.auto_place_next {
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
                let next = self
                    .picker
                    .available
                    .iter()
                    .skip(unit_idx)
                    .position(|u| u.shown() && group.is_some() && group_of(u) == group)
                    .map(|p| unit_idx + p);
                if let Some(next_idx) = next {
                    *self.state = PickerState::Placing {
                        unit_idx: next_idx,
                        preview_hex: None,
                        preview_valid: false,
                        drag_drop: false,
                    };
                } else {
                    *self.state = PickerState::Idle;
                }
            } else {
                *self.state = PickerState::Idle;
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
        *self.state = PickerState::Idle;
        Ok(None)
    }
}

/// The slip for a destination the route finder cannot reach (§5.11-§5.4:
/// movement points, terrain, zones of control, blocked hexsides): what the
/// cheapest route would cost, when there is one at any price, with the rule
/// that usually explains it for this kind of mover.
fn no_route_reason(goal: HexCoord, budget: i16, cheapest: Option<i16>, boat: bool) -> String {
    let why = if boat {
        "going upstream a gunboat has its smaller upstream allowance, §5.24"
    } else {
        "Rough and Swamp cost 3 MP a hex, §5.11"
    };
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

/// Price a goal the route finder could not reach within the move's budget:
/// the cheapest route's cost at any price (`None`: no route at all), and the
/// MP this move has for that route -- a gunboat's upstream allowance once
/// the route runs upstream (§5.24), else `budget`.
fn price_unreachable(
    game_map: &GameMap,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    placed: &PlacedUnit,
    (start, goal): (HexCoord, HexCoord),
    budget: i16,
    gunboat: Option<GunboatBudget>,
    game_state: Option<&crate::GameStateResource>,
) -> (Option<i16>, i16) {
    let Some((route, cost)) = cheapest_route(
        game_map,
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
    let have = match (gunboat, game_state) {
        (Some(gunboat), Some(gs)) => {
            let mut from = start;
            let upstream = route.iter().any(|&hex| {
                let up = step_is_upstream(&gs.0, from, hex);
                from = hex;
                up
            });
            budget.min(gunboat.left(upstream))
        }
        _ => budget,
    };
    (Some(cost), have)
}

/// Why no route reaches `goal`: when the goal itself may not hold one of the
/// `movers` (§5.51-§5.53 stacking: another tribe's stack, a leader outside
/// his command, a full hex) the route finder never even tries it -- say so,
/// rather than blaming terrain and zones of control.
fn unroutable_reason(
    gs: Option<&omdurman_rules::effects::GameState>,
    movers: &[omdurman_rules::UnitId],
    goal: HexCoord,
    budget: i16,
    cheapest: Option<i16>,
) -> String {
    // The goal is no ground this unit can stand on: gunboats keep to the
    // Nile, land units off it (§5.22).
    let wrong_ground = gs.and_then(|gs| {
        movers.iter().find_map(|&id| {
            let mover = gs.find_unit(id)?;
            let is_boat = mover.profile.kind.is_boat();
            (!gs.on_deployable_terrain(goal, is_boat)).then_some(is_boat)
        })
    });
    if let Some(is_boat) = wrong_ground {
        return if is_boat {
            format!("{goal} is not on the Nile: gunboats move only along the river (§5.22).")
        } else {
            format!("{goal} is the Nile: only gunboats move on the river (§5.22).")
        };
    }
    let stacking_refusal = gs.and_then(|gs| {
        movers.iter().find_map(|&id| {
            let mover = gs.find_unit(id)?;
            gs.check_stacking(mover, goal).err()
        })
    });
    match stacking_refusal {
        Some(error) => format!("Cannot end a move on {goal}: {error}."),
        // Nothing reaches the goal at any price, and it lies against a city
        // wall: the wall is what stands in the way.
        None if cheapest.is_none()
            && gs.is_some_and(|gs| {
                goal.neighbors().iter().any(|&n| {
                    gs.hexside_effective_is(goal, n, |k| k == omdurman_types::HexsideKind::Wall)
                })
            }) =>
        {
            format!(
                "No route to {goal}: the city wall is in the way. Units cross it only at a \
                 gate or a breach -- artillery can breach it (§6.63)."
            )
        }
        None => {
            let boat = gs.is_some_and(|gs| {
                movers
                    .iter()
                    .filter_map(|&id| gs.find_unit(id))
                    .any(|u| u.profile.kind.is_boat())
            });
            no_route_reason(goal, budget, cheapest, boat)
        }
    }
}

struct SelectedClick<'a> {
    state: &'a mut PickerState,
    game_map: &'a GameMap,
    remaining_mp: i16,
    forced_stop: bool,
    movement_path: &'a mut MovementPath,
}

impl SelectedClick<'_> {
    /// Plot the [`auto_route`] from `start` to `goal` through the ordinary
    /// per-leg click path, stopping at the first leg it refuses.
    fn plot_route(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        source: Entity,
        start: HexCoord,
        goal: HexCoord,
        game_state: Option<&crate::GameStateResource>,
    ) -> Result<(), String> {
        let Ok((_, placed)) = placed_units.get(source) else {
            return Ok(());
        };
        let budget = if self.forced_stop {
            0
        } else {
            self.remaining_mp
        };
        let gunboat = GunboatBudget::of(game_state, placed, self.movement_path);
        let Some(route) = auto_route(
            self.game_map,
            placed_units,
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
                self.game_map,
                placed_units,
                placed,
                (start, goal),
                budget,
                gunboat,
                game_state,
            );
            return Err(unroutable_reason(
                game_state.map(|gs| &gs.0),
                &movers,
                goal,
                have,
                cheapest,
            ));
        };
        let mut from = start;
        for hex in route {
            self.handle(placed_units, true, source, from, hex, game_state)?;
            match *self.state {
                PickerState::Selected {
                    start_coord,
                    remaining_mp,
                    forced_stop,
                    ..
                } if start_coord == hex => {
                    from = hex;
                    self.remaining_mp = remaining_mp;
                    self.forced_stop = forced_stop;
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

    fn handle(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        released: bool,
        source: Entity,
        start_coord: HexCoord,
        coord: HexCoord,
        game_state: Option<&crate::GameStateResource>,
    ) -> Result<(), String> {
        if !released {
            return Ok(());
        }
        if coord == start_coord {
            return Ok(());
        }
        let Ok((_, placed)) = placed_units.get(source) else {
            *self.state = PickerState::Idle;
            return Ok(());
        };
        // During Setup, `Selected` is focus-only (the player hits Del to return
        // the counter to the picker). There's no movement during deployment, so
        // don't build path legs -- bail without changing state.
        if game_state.is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup)) {
            return Ok(());
        }
        // A distant hex -- or a neighbour behind a wall -- plot the cheapest
        // legal route to it, leg by leg (round through a gate).
        if !start_coord.neighbors().contains(&coord)
            || hexside_blocks_step(self.game_map, start_coord, coord, game_state)
        {
            return self.plot_route(placed_units, source, start_coord, coord, game_state);
        }

        let leg = movement_leg_check(
            self.game_map,
            placed_units,
            placed,
            start_coord,
            coord,
            game_state,
        );
        let affordable = leg.cost > 0
            && self.remaining_mp >= leg.cost
            && gunboat_cap_ok(
                game_state,
                placed,
                self.movement_path,
                start_coord,
                coord,
                leg.cost,
            );

        if leg.accepted(affordable, self.forced_stop) {
            let new_remaining = self.remaining_mp - leg.cost;
            info!(
                "path leg accepted: {:?} -> {:?}, cost={}, remaining_mp={}, entering_zoc={}",
                start_coord, coord, leg.cost, new_remaining, leg.entering_enemy_zoc
            );
            // Accumulate the leg in the path resource.
            self.movement_path.legs.push((start_coord, coord));
            self.movement_path.cost_so_far += leg.cost;

            // Stay in Selected so the player can confirm or (if not pinned by
            // ZOC) keep adding legs. §5.43 forces a stop once a leg enters an
            // enemy ZOC; the flag is sticky for the rest of the selection.
            *self.state = PickerState::Selected {
                source,
                start_coord: coord,
                remaining_mp: new_remaining,
                forced_stop: self.forced_stop || leg.entering_enemy_zoc,
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
                forced_stop = self.forced_stop,
                entering_zoc = leg.entering_enemy_zoc,
                cost = leg.cost,
                remaining_mp = self.remaining_mp,
                "path leg rejected",
            );
            // A rejected click does not discard the work already plotted: the
            // selection and the accumulated legs stay, so the player can click
            // a legal continuation, Backspace-undo a leg, Enter-confirm, or
            // right-click/Delete to cancel. (Previously a single bad click —
            // e.g. an over-cap friendly pass-through hex per §5.51 — nuked the
            // whole path.)
            Ok(())
        }
    }

    /// Commit the accumulated multi-leg path as a single MoveUnit event.
    ///
    /// Called when the player confirms (Enter / UI button). The full path
    /// is sent in one event so the rules engine processes it atomically.
    pub(crate) fn commit_path(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        source: Entity,
        game_state: Option<&crate::GameStateResource>,
    ) -> Result<Option<GameEvent>, String> {
        if self.movement_path.legs.is_empty() {
            return Ok(None);
        }
        let Ok((_, placed)) = placed_units.get(source) else {
            *self.state = PickerState::Idle;
            self.movement_path.reset();
            return Ok(None);
        };

        // The final destination is the last leg's `to`.
        let final_dest = self.movement_path.legs.last().unwrap().1;
        let total_cost = self.movement_path.cost_so_far;

        // §5.51: the stacking cap applies where the mover *ends* its move.
        // Pass-through legs are plotted freely; block the commit if the final
        // hex would exceed the cap (mirroring the engine's apply-time check),
        // and keep the selection so the player can route on past the hex, undo
        // a leg, or cancel — instead of committing a move the engine would
        // reject on echo (animate then snap back).
        if let (Some(uid), Some(gs)) = (placed.unit_id, game_state)
            && let Some(mover) = gs.0.find_unit(uid)
            && let Err(error) = gs.0.check_stacking(mover, final_dest)
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
        let path: Vec<HexCoord> = self.movement_path.legs.iter().map(|&(_, to)| to).collect();

        info!(
            section_name = %placed.section_name,
            legs = path.len(),
            total_cost,
            "committing path"
        );

        self.movement_path.reset();
        *self.state = PickerState::Idle;

        Ok(Some(GameEvent::MoveUnit {
            sprite: omdurman_types::SpriteRef {
                section_name: placed.section_name,
                col: placed.col,
                row: placed.row,
            },
            to_q: final_dest.q,
            to_r: final_dest.r,
            cost: MovementPoints::new(total_cost),
            path,
        }))
    }
}

/// Borrowed context for resolving a click while a whole stack is selected
/// (movement-phase double-click). Mirrors [`SelectedClick`] but tracks a
/// per-unit movement budget: a leg is accepted if *any* unit can afford it,
/// and only the affordable units are charged. Slower units stop (drop) at
/// their last affordable hex; [`commit_path`](Self::commit_path) turns the one
/// plotted path into one `MoveUnit` per unit, each along the longest prefix of
/// the path its budget covers -- so after the move every unit is independent.
struct SelectedStackClick<'a, 'w, 's> {
    state: &'a mut PickerState,
    game_map: &'a GameMap,
    commands: &'a mut Commands<'w, 's>,
    remaining_mp: Vec<i16>,
    initial_mp: Vec<i16>,
    forced_stop: bool,
    movement_path: &'a mut MovementPath,
}

impl SelectedStackClick<'_, '_, '_> {
    /// Stack variant of [`SelectedClick::plot_route`]: routed with the
    /// fastest unit's budget; slower units drop off along it as usual.
    fn plot_route(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        sources: &[Entity],
        start: HexCoord,
        goal: HexCoord,
        game_state: Option<&crate::GameStateResource>,
    ) -> Result<(), String> {
        let Some(Ok((_, placed))) = sources.first().map(|&s| placed_units.get(s)) else {
            return Ok(());
        };
        let budget = if self.forced_stop {
            0
        } else {
            self.remaining_mp.iter().copied().max().unwrap_or(0)
        };
        let gunboat = GunboatBudget::of(game_state, placed, self.movement_path);
        let Some(route) = auto_route(
            self.game_map,
            placed_units,
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
                .filter_map(|&s| placed_units.get(s).ok().and_then(|(_, p)| p.unit_id))
                .collect();
            let (cheapest, have) = price_unreachable(
                self.game_map,
                placed_units,
                placed,
                (start, goal),
                budget,
                gunboat,
                game_state,
            );
            return Err(unroutable_reason(
                game_state.map(|gs| &gs.0),
                &movers,
                goal,
                have,
                cheapest,
            ));
        };
        let mut from = start;
        for hex in route {
            self.handle(placed_units, true, sources, from, hex, game_state)?;
            match &*self.state {
                PickerState::SelectedStack(sel) if sel.start_coord == hex => {
                    from = hex;
                    self.remaining_mp = sel.remaining_mp.clone();
                    self.forced_stop = sel.forced_stop;
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

    fn handle(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        released: bool,
        sources: &[Entity],
        start_coord: HexCoord,
        coord: HexCoord,
        game_state: Option<&crate::GameStateResource>,
    ) -> Result<(), String> {
        if !released {
            return Ok(());
        }
        if coord == start_coord {
            return Ok(());
        }
        let Ok((_, placed)) = placed_units.get(sources[0]) else {
            *self.state = PickerState::Idle;
            return Ok(());
        };
        // No movement during Setup (the stack selection itself is movement
        // phase only, but a stale state could outlive a phase change).
        if game_state.is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup)) {
            return Ok(());
        }
        // A distant hex -- or a neighbour behind a wall -- plot the cheapest
        // legal route to it, leg by leg (round through a gate).
        if !start_coord.neighbors().contains(&coord)
            || hexside_blocks_step(self.game_map, start_coord, coord, game_state)
        {
            return self.plot_route(placed_units, sources, start_coord, coord, game_state);
        }

        // Stacking pre-check uses the first unit as the group's representative
        // (the engine re-validates each unit's move at commit, so this is a
        // courtesy gate for the common case -- moving a stack into a hex that
        // already pushes past the §5.51 cap).
        let leg = movement_leg_check(
            self.game_map,
            placed_units,
            placed,
            start_coord,
            coord,
            game_state,
        );
        // A leg is affordable if at least one unit's budget covers it. Units
        // that can't afford it keep their remaining (they are dropped here).
        let affordable = leg.cost > 0
            && self.remaining_mp.iter().any(|&mp| mp >= leg.cost)
            && gunboat_cap_ok(
                game_state,
                placed,
                self.movement_path,
                start_coord,
                coord,
                leg.cost,
            );

        if leg.accepted(affordable, self.forced_stop) {
            let new_remaining: Vec<i16> = self
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
                    .zip(&self.remaining_mp)
                    .filter(|(nr, mp)| **nr < **mp)
                    .count(),
                leg.entering_enemy_zoc,
            );
            self.movement_path.legs.push((start_coord, coord));
            self.movement_path.cost_so_far += leg.cost;
            *self.state = PickerState::SelectedStack(StackSelection {
                sources: sources.to_vec(),
                start_coord: coord,
                remaining_mp: new_remaining,
                initial_mp: self.initial_mp.clone(),
                forced_stop: self.forced_stop || leg.entering_enemy_zoc,
            });
            Ok(())
        } else {
            info!(
                adjacent = leg.adjacent,
                enemy_occupied = leg.enemy_occupied,
                passable = leg.passable,
                affordable,
                stacking_ok = leg.stacking_ok,
                forced_stop = self.forced_stop,
                entering_zoc = leg.entering_enemy_zoc,
                cost = leg.cost,
                "stack path leg rejected",
            );
            // Keep the group and the plotted path: a rejected click (an
            // over-cap friendly pass-through hex per §5.51, a non-adjacent or
            // out-of-budget hex, or a post-ZOC stop) must not discard the route
            // already plotted. The player can click a legal continuation,
            // Backspace-undo, Enter-confirm, or right-click/Delete to cancel.
            Ok(())
        }
    }

    /// Commit the plotted path as one `MoveUnit` per unit, each along the
    /// longest prefix of the path its remaining budget covers.
    ///
    /// Units whose budget can't reach even the first hex stay put (no event).
    /// This is what "lower-movement units are dropped along the path": every
    /// unit stops at its last affordable hex, and afterwards the units are
    /// plain independent counters again.
    pub(crate) fn commit_path(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        sources: &[Entity],
        game_state: Option<&crate::GameStateResource>,
    ) -> Vec<GameEvent> {
        if self.movement_path.legs.is_empty() {
            return Vec::new();
        }
        let mut events = Vec::new();

        for (i, &source) in sources.iter().enumerate() {
            // The unit's budget for the whole plotted path is what it had when
            // the stack was selected: `remaining_mp` has already been charged
            // for the plotted legs.
            let remaining = self.initial_mp[i];
            let Ok((_, placed)) = placed_units.get(source) else {
                continue;
            };
            // Longest prefix whose cumulative terrain cost fits the budget.
            let mut cum = 0i16;
            let mut prefix: Vec<HexCoord> = Vec::new();
            for &(from, to) in &self.movement_path.legs {
                let leg_cost = floor_movement_cost(
                    self.game_map,
                    from,
                    to,
                    placed.is_boat,
                    game_state.map(|gs| &gs.0),
                );
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
            let stacking_ok = match (placed.unit_id, game_state) {
                (Some(uid), Some(gs)) if let Some(mover) = gs.0.find_unit(uid) => {
                    gs.0.check_stacking(mover, to).is_ok()
                }
                _ => true,
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
                cost: MovementPoints::new(cum),
                path: prefix,
            });
        }

        // The group move is over: the stack is deselected and each unit is an
        // independent counter again. If nothing was committed (every unit was
        // budget-dropped or stacking-skipped), keep the plot so the player can
        // extend or cancel instead of silently losing it.
        if events.is_empty() && !self.movement_path.legs.is_empty() {
            warn!("stack commit produced no moves; keeping the plotted path");
            return events;
        }
        for &source in sources {
            self.commands.entity(source).remove::<Selected>();
        }
        self.movement_path.reset();
        *self.state = PickerState::Idle;
        events
    }
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
    game_state: Option<Res<crate::GameStateResource>>,
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
    if game_state
        .as_deref()
        .is_some_and(|gs| !peers.may_act_now(&gs.0))
    {
        return;
    }
    crate::ui_trace::path_confirmed(
        picker_ctx.movement_path.legs.len(),
        picker_ctx.movement_path.cost_so_far,
        &crate::ui_trace::Stamp::of(game_state.as_deref()),
    );
    match ActiveSelection::snapshot(&picker_ctx.state) {
        ActiveSelection::Single { source, .. } => {
            let mut sel = SelectedClick {
                state: &mut picker_ctx.state,
                game_map: &picker_ctx.game_map,
                remaining_mp: 0,
                forced_stop: false,
                movement_path: &mut picker_ctx.movement_path,
            };
            match sel.commit_path(&picker_ctx.placed_units, source, game_state.as_deref()) {
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
                        dispatches.push(crate::submit::REFUSED_HEADER, reason);
                    }
                }
            }
        }
        ActiveSelection::Stack(sel) => {
            let mut sel_click = SelectedStackClick {
                state: &mut picker_ctx.state,
                game_map: &picker_ctx.game_map,
                commands: &mut picker_ctx.commands,
                remaining_mp: sel.remaining_mp.clone(),
                initial_mp: sel.initial_mp.clone(),
                forced_stop: sel.forced_stop,
                movement_path: &mut picker_ctx.movement_path,
            };
            for event in sel_click.commit_path(
                &picker_ctx.placed_units,
                &sel.sources,
                game_state.as_deref(),
            ) {
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
    game_state: Option<Res<crate::GameStateResource>>,
    peers: crate::peers::Peers,
) {
    if !commanded(&mut commands_in, PickerCommand::UndoStep) {
        return;
    }
    if picker_ctx.movement_path.legs.is_empty() {
        return;
    }
    // Only the owning player may act.
    if game_state
        .as_deref()
        .is_some_and(|gs| !peers.may_act_now(&gs.0))
    {
        return;
    }
    crate::ui_trace::path_undone(
        picker_ctx.movement_path.legs.len() - 1,
        &crate::ui_trace::Stamp::of(game_state.as_deref()),
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
    let cost = floor_movement_cost(
        &picker_ctx.game_map,
        from,
        to,
        is_boat,
        game_state.as_deref().map(|gs| &gs.0),
    );
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
    game_state: Option<Res<crate::GameStateResource>>,
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
    let remaining_mp = unit_remaining_mp(game_state.as_deref(), placed);
    let previous: Vec<Entity> = match &*picker_ctx.state {
        PickerState::Selected { source, .. } => vec![*source],
        PickerState::SelectedStack(sel) => sel.sources.clone(),
        PickerState::SelectedTile(sel) => sel.sources.clone(),
        _ => Vec::new(),
    };
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
    game_state: Option<Res<crate::GameStateResource>>,
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
    let Some(gs) = game_state.as_deref() else {
        return;
    };
    if !matches!(gs.0.phase, omdurman_rules::Phase::Setup) {
        return;
    }
    // The side deploying now (set-up is sequential, §9.111/§9.211/§9.321).
    if !peers.may_act_now(&gs.0) {
        return;
    }
    let Ok((_, placed)) = picker_ctx.placed_units.get(*source) else {
        return;
    };
    crate::ui_trace::button("remove unit (Del)");
    let submitted = submit.submit(
        &gs.0,
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
    game_state: Option<Res<crate::GameStateResource>>,
    mut paths: ResMut<UnitPaths>,
    mut movement_path: ResMut<MovementPath>,
    mut last_active: Local<Option<omdurman_types::Player>>,
) {
    let stamp = crate::ui_trace::Stamp::of(game_state.as_deref());
    let Some(gs) = game_state else { return };
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
        world.insert_resource(MovementPath {
            legs: vec![(first, HexCoord::new(12, 21))],
            cost_so_far: 3,
        });
        world
            .run_system_once(
                move |mut state: ResMut<PickerState>,
                      mut commands: Commands,
                      placed: Query<(Entity, &PlacedUnit)>,
                      mut path: ResMut<MovementPath>| {
                    handle_stack_double_click(
                        &mut state,
                        &mut commands,
                        &placed,
                        second,
                        None,
                        None,
                        &|_| true,
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

    /// Play-test repro: Ali Wad Helu ordered onto a Jaalin stack was refused
    /// as "No route ... terrain costs, zones of control"; the real reason is
    /// §5.53 stacking at the goal.
    #[test]
    fn an_illegal_goal_stack_is_reported_as_stacking_not_routing() {
        use omdurman_rules::{UnitId, UnitPlacement, unit_profiles::profile_for_unit};
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
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
        let reason = unroutable_reason(Some(&gs), &[UnitId::AliWadHelu_0_0], goal, 14, None);
        assert!(reason.contains("§5.53"), "{reason}");
        assert!(!reason.starts_with("No route"), "{reason}");
        // An empty goal keeps the routing explanation.
        let reason = unroutable_reason(
            Some(&gs),
            &[UnitId::AliWadHelu_0_0],
            HexCoord::new(9, 9),
            14,
            None,
        );
        assert!(reason.starts_with("No route"), "{reason}");
        // A route that exists at some price says what it costs.
        let reason = unroutable_reason(
            Some(&gs),
            &[UnitId::AliWadHelu_0_0],
            HexCoord::new(9, 9),
            14,
            Some(19),
        );
        assert!(reason.contains("costs 19 MP"), "{reason}");
    }
}

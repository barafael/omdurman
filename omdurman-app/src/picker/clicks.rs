//! Board click handling: the place / select / move state machine.

use super::*;

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
    pub ground: Res<'w, crate::picking::PointerGroundHit>,
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub action_writer: MessageWriter<'w, events::LocalAction>,
    pub ui_trace: ResMut<'w, crate::ui_trace::UiTrace>,
}

pub fn handle_picker_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    mut picker_ctx: PickerContext,
    game_state: Option<Res<crate::GameStateResource>>,
    peers: crate::peers::Peers,
    time: Res<Time>,
    mut last_click: Local<Option<(f64, HexCoord)>>,
) {
    let game_state = game_state.as_deref();
    let pressed = buttons.just_pressed(MouseButton::Left);
    let released = buttons.just_released(MouseButton::Left);
    if !pressed && !released {
        return;
    }
    // (Map-interaction gating is declarative: this system runs in
    // `MapPointerInputSet`, which is skipped while the pointer is over UI.)

    // §turn-order: a unit may only be moved on its owner's turn. When a game is
    // live, gate interactive movement on the local player being the rules
    // engine's active player (`handle_idle_click`/move path below). Placement
    // during set-up is not gated. With no game state (editor) there is no gate.
    let may_move = game_state.is_none_or(|gs| peers.may_act(gs.0.phase_player()));

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

    let Some(hit) = **picker_ctx.ground else {
        return;
    };
    let origin = picker_ctx
        .layout
        .adjusted_origin(&picker_ctx.overlay.params);
    let coord = hit_to_hex(hit, origin, &picker_ctx.overlay.params);
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

    // UI trace: record the board click itself (left press), with what stood
    // under the cursor, so click-through and rejected clicks are observable.
    if pressed {
        let units: Vec<String> = picker_ctx
            .placed_units
            .iter()
            .filter(|(_, u)| u.coord == coord)
            .map(|(_, u)| crate::ui_trace::placed_label(u, game_state))
            .collect();
        picker_ctx.ui_trace.record(
            time.elapsed_secs_f64(),
            game_state.map(|gs| gs.0.current_turn.value()),
            game_state.map(|gs| format!("{:?}", gs.0.phase)),
            crate::ui_trace::UiTraceEvent::BoardClick {
                hex: crate::ui_trace::HexLabel::of(coord),
                units,
            },
        );
    }

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
                return; // not your unit
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
                return; // another member's command
            }
            picker_ctx.commands.entity(entity).insert(Selected);
            *picker_ctx.state = PickerState::Selected {
                source: entity,
                start_coord: coord,
                remaining_mp: 0,
                forced_stop: false,
            };
            return;
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
        let now = time.elapsed_secs_f64();
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
                );
            }
        }
        return;
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
        // During deployment in a *bound* game, a unit may only be placed inside
        // its owner's deployment zone (§9.2/§9.3). We gate the *click* on the
        // same engine predicate the deployment overlay is drawn from, so the UI
        // can't commit an out-of-zone `PlaceUnit`. (Placement otherwise isn't
        // phase-gated.) An unbound session (empty faction binding) is exempt:
        // placement is free at all valid hexes in every phase, and the zone
        // rings there are display-only.
        ActiveSelection::Placing { unit_idx, .. }
            if peers.any_assigned()
                && game_state
                    .is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup))
                && !deploy_hex_allowed(game_state, &picker_ctx.picker, unit_idx, coord) =>
        {
            // Off-zone: ignore the click, keep the unit in hand.
        }
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
            if let Some(event) = placing.handle(
                &picker_ctx.placed_units,
                released,
                unit_idx,
                drag_drop,
                coord,
                game_state,
            ) {
                picker_ctx
                    .action_writer
                    .write(events::LocalAction { event });
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
                overlay: &picker_ctx.overlay,
                game_map: &picker_ctx.game_map,
                commands: &mut picker_ctx.commands,
                origin,
                remaining_mp,
                forced_stop,
                movement_path: &mut picker_ctx.movement_path,
            };
            if let Some(event) = sel.handle(
                &picker_ctx.placed_units,
                released,
                source,
                start_coord,
                coord,
                game_state,
            ) {
                info!("writing LocalAction for MoveUnit");
                picker_ctx
                    .action_writer
                    .write(events::LocalAction { event });
            }
            if matches!(&*picker_ctx.state, PickerState::Idle) {
                picker_ctx.commands.entity(source).remove::<Selected>();
            }
        }
        // Outside the movement phase a single-unit selection is an *action*
        // target, not a mover: fire clicks are consumed earlier by
        // `handle_fire_allocation_click` (registered `.before` this system),
        // so there is nothing to plot here -- keep the selection so the fire
        // overlay stays active (§6.41).
        ActiveSelection::Single { .. } => {}
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
                overlay: &picker_ctx.overlay,
                game_map: &picker_ctx.game_map,
                commands: &mut picker_ctx.commands,
                origin,
                remaining_mp: sel.remaining_mp.clone(),
                initial_mp: sel.initial_mp.clone(),
                forced_stop: sel.forced_stop,
                movement_path: &mut picker_ctx.movement_path,
            };
            if let Some(event) = sel_click.handle(
                &picker_ctx.placed_units,
                released,
                &sel.sources,
                sel.start_coord,
                coord,
                game_state,
            ) {
                picker_ctx
                    .action_writer
                    .write(events::LocalAction { event });
            }
        }
        // A stale movement stack in a non-movement phase: fire clicks are
        // consumed by `handle_fire_allocation_click`, so nothing to plot.
        ActiveSelection::Stack(_) => {}
        // The combat tile selection (double-click in a fire sub-phase or
        // Melee). Only *presses* act — the release of the selecting
        // double-click lands here too, and acting on it would tear the tile
        // down the same frame it was made. A press on the tile's own hex
        // dismisses it; a press on another hex switches to a single-counter
        // selection there when one is pickable, and otherwise *keeps* the
        // tile — combat target clicks pass through here on their press and
        // are consumed only on release (by `handle_fire_allocation_click` /
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
    commands.entity(entity).insert(Selected);
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
/// leak onto an unrelated counter.
fn handle_stack_double_click(
    state: &mut PickerState,
    commands: &mut Commands,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    coord: HexCoord,
    game_state: Option<&crate::GameStateResource>,
    restrict_to: Option<omdurman_types::Player>,
    scope_ok: &dyn Fn(&omdurman_rules::UnitIdentity) -> bool,
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
        commands.entity(e).insert(Selected);
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
        commands.entity(e).insert(Selected);
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

/// Whether the picker unit at `unit_idx` may be deployed on `coord` during
/// setup: `coord` must lie in that unit's owner's deployment zone (§9.2/§9.3).
/// Owner and boat-ness are derived from the sprite via [`deploy_candidate`]
/// (i.e. `profile.kind.is_boat()`) -- the same source of truth the engine's
/// `can_deploy_unit` uses -- so this gate can never disagree with the preview
/// or the apply path about whether a counter is a gunboat (the cached
/// `PickerUnit.is_boat` flag is not reliable for this). Returns `true` when
/// there is no game state or the sprite can't be resolved, so non-setup /
/// unbound placement is never blocked by this gate.
fn deploy_hex_allowed(
    game_state: Option<&crate::GameStateResource>,
    picker: &UnitPicker,
    unit_idx: usize,
    coord: HexCoord,
) -> bool {
    let Some(gs) = game_state else { return true };
    let Some(candidate) = deploy_candidate(picker, unit_idx, coord) else {
        return true;
    };
    gs.0.in_deployment_zone(
        candidate.profile.identity.owner(),
        coord,
        candidate.profile.kind.is_boat(),
    )
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
    ) -> Option<GameEvent> {
        if released && !drag_drop {
            *self.state = PickerState::Placing {
                unit_idx,
                preview_hex: None,
                preview_valid: false,
                drag_drop: false,
            };
            return None;
        }

        let Some(unit) = self.picker.available.get(unit_idx) else {
            *self.state = PickerState::Idle;
            return None;
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
        let can_place = if let Some(gs) = game_state {
            if matches!(gs.0.phase, omdurman_rules::Phase::Setup) {
                game_state
                    .zip(deploy_candidate(self.picker, unit_idx, coord))
                    .is_some_and(|(gs, candidate)| gs.0.can_deploy_unit(&candidate).is_ok())
            } else if matches!(gs.0.phase, omdurman_rules::Phase::Movement) {
                deploy_candidate(self.picker, unit_idx, coord).is_some_and(|candidate| {
                    gs.0.can_place_single_reinforcement(&candidate).is_ok()
                })
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

            spawn_placed_unit(
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
                let section = unit.section_name;
                let next = self
                    .picker
                    .available
                    .iter()
                    .skip(unit_idx)
                    .position(|u| u.visible && u.section_name == section)
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
            return Some(GameEvent::PlaceUnit {
                sprite: omdurman_types::SpriteRef {
                    section_name: unit.section_name,
                    col: unit.col,
                    row: unit.row,
                },
                coord: omdurman_types::HexCoord::new(coord.q, coord.r),
                is_boat,
            });
        }
        *self.state = PickerState::Idle;
        None
    }
}

struct SelectedClick<'a, 'w, 's> {
    state: &'a mut PickerState,
    overlay: &'a HexOverlay,
    game_map: &'a GameMap,
    commands: &'a mut Commands<'w, 's>,
    origin: Vec2,
    remaining_mp: i16,
    forced_stop: bool,
    movement_path: &'a mut MovementPath,
}

impl SelectedClick<'_, '_, '_> {
    fn handle(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        released: bool,
        source: Entity,
        start_coord: HexCoord,
        coord: HexCoord,
        game_state: Option<&crate::GameStateResource>,
    ) -> Option<GameEvent> {
        if !released {
            return None;
        }
        if coord == start_coord {
            return None;
        }
        let Ok((_, placed)) = placed_units.get(source) else {
            *self.state = PickerState::Idle;
            return None;
        };
        // During Setup, `Selected` is focus-only (the player hits Del to return
        // the counter to the picker). There's no movement during deployment, so
        // don't build path legs -- bail without changing state.
        if game_state.is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup)) {
            return None;
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
            None
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
            None
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
    ) -> Option<GameEvent> {
        if self.movement_path.legs.is_empty() {
            return None;
        }
        let Ok((_, placed)) = placed_units.get(source) else {
            *self.state = PickerState::Idle;
            self.movement_path.reset();
            return None;
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
            && gs.0.check_stacking(mover, final_dest).is_err()
        {
            warn!(
                section_name = %placed.section_name,
                final_dest.q = final_dest.q,
                final_dest.r = final_dest.r,
                "commit rejected: final destination breaks the §5.51 stacking cap",
            );
            return None;
        }

        // Animate through each leg sequentially.
        let origin = self.origin;
        let overlay = self.overlay;
        for &(from, to) in &self.movement_path.legs {
            let from_pos = hex_world_pos(from, origin, &overlay.params);
            let to_pos = hex_world_pos(to, origin, &overlay.params);
            self.commands.entity(source).insert(MovementAnimation {
                from: Vec3::new(from_pos.x, UNIT_HEIGHT, from_pos.z),
                to: Vec3::new(to_pos.x, UNIT_HEIGHT, to_pos.z),
                progress: 0.0,
                target_coord: to,
            });
        }

        let path: Vec<HexCoord> = self.movement_path.legs.iter().map(|&(_, to)| to).collect();

        info!(
            section_name = %placed.section_name,
            legs = path.len(),
            total_cost,
            "committing path"
        );

        self.movement_path.reset();
        *self.state = PickerState::Idle;

        Some(GameEvent::MoveUnit {
            sprite: omdurman_types::SpriteRef {
                section_name: placed.section_name,
                col: placed.col,
                row: placed.row,
            },
            to_q: final_dest.q,
            to_r: final_dest.r,
            cost: MovementPoints::new(total_cost),
            path,
        })
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
    overlay: &'a HexOverlay,
    game_map: &'a GameMap,
    commands: &'a mut Commands<'w, 's>,
    origin: Vec2,
    remaining_mp: Vec<i16>,
    initial_mp: Vec<i16>,
    forced_stop: bool,
    movement_path: &'a mut MovementPath,
}

impl SelectedStackClick<'_, '_, '_> {
    fn handle(
        &mut self,
        placed_units: &Query<(Entity, &PlacedUnit)>,
        released: bool,
        sources: &[Entity],
        start_coord: HexCoord,
        coord: HexCoord,
        game_state: Option<&crate::GameStateResource>,
    ) -> Option<GameEvent> {
        if !released {
            return None;
        }
        if coord == start_coord {
            return None;
        }
        let Ok((_, placed)) = placed_units.get(sources[0]) else {
            *self.state = PickerState::Idle;
            return None;
        };
        // No movement during Setup (the stack selection itself is movement
        // phase only, but a stale state could outlive a phase change).
        if game_state.is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup)) {
            return None;
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
            None
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
            None
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
        let start_coord = self.movement_path.legs[0].0;
        let origin = self.origin;
        let overlay = self.overlay;
        let mut events = Vec::new();

        for (i, &source) in sources.iter().enumerate() {
            let remaining = self.remaining_mp[i];
            let Ok((_, placed)) = placed_units.get(source) else {
                continue;
            };
            // Longest prefix whose cumulative terrain cost fits the budget.
            let mut cum = 0i16;
            let mut prefix: Vec<HexCoord> = Vec::new();
            for &(from, to) in &self.movement_path.legs {
                let leg_cost = floor_movement_cost(self.game_map, from, to, placed.is_boat);
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
            // Animate this unit's final hop (mirrors the single-unit commit:
            // the engine sets the authoritative position on the echo).
            let from_coord = if prefix.len() >= 2 {
                prefix[prefix.len() - 2]
            } else {
                start_coord
            };
            let from_pos = hex_world_pos(from_coord, origin, &overlay.params);
            let to_pos = hex_world_pos(to, origin, &overlay.params);
            self.commands.entity(source).insert(MovementAnimation {
                from: Vec3::new(from_pos.x, UNIT_HEIGHT, from_pos.z),
                to: Vec3::new(to_pos.x, UNIT_HEIGHT, to_pos.z),
                progress: 0.0,
                target_coord: to,
            });
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
    time: Res<Time>,
    mut movement_path: ResMut<MovementPath>,
    mut ui_trace: ResMut<crate::ui_trace::UiTrace>,
) {
    if matches!(&*state, PickerState::Idle) && !movement_path.legs.is_empty() {
        ui_trace.record(
            time.elapsed_secs_f64(),
            None,
            None,
            crate::ui_trace::UiTraceEvent::PathCleared {
                reason: "selection dropped",
            },
        );
        movement_path.reset();
    }
}

/// Confirm a pending movement path when the player presses Enter.
///
/// Reads keyboard input and, if a path is pending, fires the commit.
pub(crate) fn confirm_movement_path(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut picker_ctx: PickerContext,
    game_state: Option<Res<crate::GameStateResource>>,
    peers: crate::peers::Peers,
) {
    if !keys.just_pressed(KeyCode::Enter) {
        return;
    }
    if picker_ctx.movement_path.legs.is_empty() {
        return;
    }
    // Must be the owning player's turn.
    if game_state
        .as_deref()
        .is_some_and(|gs| !peers.may_act(gs.0.phase_player()))
    {
        return;
    }
    picker_ctx.ui_trace.record(
        time.elapsed_secs_f64(),
        game_state.as_deref().map(|gs| gs.0.current_turn.value()),
        game_state.as_deref().map(|gs| format!("{:?}", gs.0.phase)),
        crate::ui_trace::UiTraceEvent::PathConfirmed {
            legs: picker_ctx.movement_path.legs.len(),
            cost: picker_ctx.movement_path.cost_so_far,
        },
    );
    let origin = picker_ctx
        .layout
        .adjusted_origin(&picker_ctx.overlay.params);
    match ActiveSelection::snapshot(&picker_ctx.state) {
        ActiveSelection::Single { source, .. } => {
            let mut sel = SelectedClick {
                state: &mut picker_ctx.state,
                overlay: &picker_ctx.overlay,
                game_map: &picker_ctx.game_map,
                commands: &mut picker_ctx.commands,
                origin,
                remaining_mp: 0,
                forced_stop: false,
                movement_path: &mut picker_ctx.movement_path,
            };
            if let Some(event) =
                sel.commit_path(&picker_ctx.placed_units, source, game_state.as_deref())
            {
                picker_ctx
                    .action_writer
                    .write(events::LocalAction { event });
            }
        }
        ActiveSelection::Stack(sel) => {
            let mut sel_click = SelectedStackClick {
                state: &mut picker_ctx.state,
                overlay: &picker_ctx.overlay,
                game_map: &picker_ctx.game_map,
                commands: &mut picker_ctx.commands,
                origin,
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

/// Undo the last leg of the pending movement path when the player presses
/// Backspace. Refunds the leg's movement points and steps the planned position
/// back one hex; the path can then be re-extended or committed. Only acts on
/// the owning player's turn, while a unit is selected with a pending path.
pub(crate) fn undo_movement_leg(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut picker_ctx: PickerContext,
    game_state: Option<Res<crate::GameStateResource>>,
    peers: crate::peers::Peers,
) {
    if !keys.just_pressed(KeyCode::Backspace) {
        return;
    }
    if picker_ctx.movement_path.legs.is_empty() {
        return;
    }
    // Only the owning player may act.
    if game_state
        .as_deref()
        .is_some_and(|gs| !peers.may_act(gs.0.phase_player()))
    {
        return;
    }
    picker_ctx.ui_trace.record(
        time.elapsed_secs_f64(),
        game_state.as_deref().map(|gs| gs.0.current_turn.value()),
        game_state.as_deref().map(|gs| format!("{:?}", gs.0.phase)),
        crate::ui_trace::UiTraceEvent::PathUndone {
            legs_left: picker_ctx.movement_path.legs.len() - 1,
        },
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
    let cost = floor_movement_cost(&picker_ctx.game_map, from, to, is_boat);
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

/// Return the focused unit to the picker when the player presses Delete, but
/// only during the placement phase ([`Phase::Setup`]) -- a unit placed in a
/// prior phase is not removable. Mirrors the engine's `RemoveDeployedUnit`
/// gate (§9.2/§9.3). The engine re-validates phase + ownership on apply.
pub(crate) fn delete_selected_unit(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut picker_ctx: PickerContext,
    game_state: Option<Res<crate::GameStateResource>>,
    peers: crate::peers::Peers,
    mut pending: Option<ResMut<crate::PendingEdits>>,
) {
    if !keys.just_pressed(KeyCode::Delete) {
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
    if !peers.may_act(gs.0.phase_player()) {
        return;
    }
    let Ok((_, placed)) = picker_ctx.placed_units.get(*source) else {
        return;
    };
    let Some(ref mut pending) = pending else {
        return;
    };
    picker_ctx.ui_trace.record(
        time.elapsed_secs_f64(),
        Some(gs.0.current_turn.value()),
        Some(format!("{:?}", gs.0.phase)),
        crate::ui_trace::UiTraceEvent::Button {
            id: "remove unit (Del)",
        },
    );
    pending.submit_game(GameEvent::RemoveUnit {
        sprite: omdurman_types::SpriteRef {
            section_name: placed.section_name,
            col: placed.col,
            row: placed.row,
        },
    });
    // Deselect; the apply path despawns the entity and returns the counter to
    // the picker.
    picker_ctx.commands.entity(*source).remove::<Selected>();
    *picker_ctx.state = PickerState::Idle;
}

pub fn cancel_placement(
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut state: ResMut<PickerState>,
    mut movement_path: ResMut<MovementPath>,
    mut ui_trace: ResMut<crate::ui_trace::UiTrace>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    // (Runs in `MapPointerInputSet` -- skipped while the pointer is over UI.)
    if !movement_path.legs.is_empty() {
        ui_trace.record(
            time.elapsed_secs_f64(),
            None,
            None,
            crate::ui_trace::UiTraceEvent::PathCleared {
                reason: "right-click cancel",
            },
        );
    }
    movement_path.reset();
    *state = PickerState::Idle;
}

/// Clear every unit's movement path when the active player changes -- i.e. at
/// the end of a player's turn -- so the arrows show the moves made *this* turn
/// and reset for the next. The turn lives in the rules engine
/// (`GameState.active_player`); we watch it via a `Local` snapshot rather than a
/// dedicated event, so the reset also fires correctly on replay and snapshot
/// convergence, where no local "end turn" click occurs.
pub fn clear_paths_on_turn_change(
    game_state: Option<Res<crate::GameStateResource>>,
    time: Res<Time>,
    mut paths: ResMut<UnitPaths>,
    mut movement_path: ResMut<MovementPath>,
    mut last_active: Local<Option<omdurman_types::Player>>,
    mut ui_trace: ResMut<crate::ui_trace::UiTrace>,
) {
    let Some(gs) = game_state else { return };
    let active = gs.0.active_player;
    if *last_active != Some(active) {
        if last_active.is_some() {
            ui_trace.record(
                time.elapsed_secs_f64(),
                Some(gs.0.current_turn.value()),
                Some(format!("{:?}", gs.0.phase)),
                crate::ui_trace::UiTraceEvent::PathCleared {
                    reason: "turn change",
                },
            );
            paths.0.clear();
            movement_path.reset();
        }
        *last_active = Some(active);
    }
}

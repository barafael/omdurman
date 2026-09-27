//! Board overlays: preview, movement rings, selection outline, hover,
//! path arrows and shadows.

use super::*;

/// Marker for one arrow segment of a unit's movement path. The whole arrow set
/// is rebuilt (see [`movement_path_arrows`]) whenever the paths or the hovered
/// hex change, and each segment is spawned with the dim or bright material
/// already chosen for its path's hover state -- so no per-segment path data
/// needs to live on the entity.
#[derive(Component)]
pub(crate) struct MovementPathArrow;

/// Bundle of the picker + picker-state + game-map resources consumed by
/// [`placement_preview_mesh`], so the system stays under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PickerPlacementState<'w> {
    pub picker: Res<'w, UnitPicker>,
    pub state: ResMut<'w, PickerState>,
    pub game_map: Res<'w, GameMap>,
}

/// Bundle of the hex-layout + overlay + game-map + cameras used by
/// [`movement_path_labels`] and other picker mesh systems, so their signatures
/// stay under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct HexMapView<'w, 's> {
    pub layout: Res<'w, HexLayout>,
    pub overlay: Res<'w, HexOverlay>,
    pub game_map: Res<'w, GameMap>,
    pub cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<RtsCamera>>,
}

/// Bundle of the game-map + optional game-state consumed by
/// [`movement_overlay_mesh`], so its signature stays under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct MovementOverlayCtx<'w> {
    pub game_map: Res<'w, GameMap>,
    pub game_state: Option<Res<'w, crate::GameStateResource>>,
}

/// Bundle of the three movement-ring marker queries (green reachable, gray
/// range, yellow ZOC) so [`movement_overlay_mesh`] stays under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct MovementRingQueries<'w, 's> {
    pub existing_green: Query<'w, 's, Entity, With<MovementHexRing>>,
    pub existing_gray: Query<'w, 's, Entity, With<MovementRangeRing>>,
    pub existing_zoc: Query<'w, 's, Entity, With<MovementZocRing>>,
}

/// Bundle of the read-only picker state + placed-units query consumed by
/// [`movement_overlay_mesh`], so the system stays under Bevy's system-parameter
/// limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PickerReadSelection<'w, 's> {
    pub state: Res<'w, PickerState>,
    pub placed_units: Query<'w, 's, (Entity, &'static PlacedUnit)>,
}

#[derive(Component)]
pub(crate) struct PreviewHexRing;

pub fn placement_preview_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    picker_state: PickerPlacementState,
    ground: Res<crate::picking::PointerGroundHit>,
    placed_units: Query<&PlacedUnit>,
    existing: Query<Entity, With<PreviewHexRing>>,
    game_state: Option<Res<crate::GameStateResource>>,
) {
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let PickerPlacementState {
        picker,
        mut state,
        game_map,
    } = picker_state;
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);

    let PickerState::Placing {
        unit_idx,
        preview_hex,
        preview_valid,
        ..
    } = &mut *state
    else {
        return;
    };

    let Some(unit) = picker.available.get(*unit_idx) else {
        *preview_hex = None;
        return;
    };

    let Some(hit) = **ground else {
        *preview_hex = None;
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    if !game_map.hexes.contains_key(&coord) {
        *preview_hex = None;
        return;
    }

    // Gate the preview on the same engine predicate the click and the apply
    // path use (phase + zone + full stacking, §9.2/§9.3), so the ring never
    // shows green for a hex the engine would reject. Editor / unbound non-setup
    // placement falls back to passable-and-vacant.
    let valid = match game_state.as_deref() {
        Some(gs) if matches!(gs.0.phase, omdurman_rules::Phase::Setup) => {
            deploy_candidate(&picker, *unit_idx, coord)
                .is_some_and(|candidate| gs.0.can_deploy_unit(&candidate).is_ok())
        }
        _ => {
            let occupied = placed_units.iter().any(|u| u.coord == coord);
            !occupied && coord_passable(&game_map, coord, unit.is_boat)
        }
    };
    *preview_hex = Some(coord);
    *preview_valid = valid;

    let pos = hex_world_pos(coord, origin, &overlay.params);
    let material = if valid {
        assets.green.clone()
    } else {
        assets.red.clone()
    };

    commands.spawn((
        PreviewHexRing,
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(overlay.params.hex_size)),
        Visibility::Visible,
    ));
}

/// Tint the cursor hex marker (`SelectionMarker`) by placement legality while
/// a unit is in hand: green on a legal deploy hex, red on an illegal one (and
/// red when idle / not placing). `preview_valid` is maintained by
/// [`placement_preview_mesh`], which must run first -- hence the `.after(...)`.
pub(crate) fn placement_marker_color(
    state: Res<PickerState>,
    assets: Res<HexRingAssets>,
    mut marker: Query<&mut MeshMaterial3d<StandardMaterial>, With<crate::render::SelectionMarker>>,
) {
    let valid = match &*state {
        PickerState::Placing { preview_valid, .. } => *preview_valid,
        _ => false,
    };
    let Ok(mut mat) = marker.single_mut() else {
        return;
    };
    mat.0 = if valid {
        assets.marker_green.clone()
    } else {
        assets.marker_red.clone()
    };
}

// -- Click handling: placement + movement ---------------------------------------

/// Render per-hex incremental cost labels along the pending movement path.
///
/// For each leg `(from, to)`, a small egui label showing the terrain cost
/// is rendered at the world-space position of `to`, projected to screen
/// space.  For gunboat units, the label is prefixed with ↑/↓ to indicate
/// upstream/downstream direction (§5.24).
/// Labels are only shown while a path is being built (non-empty
/// `MovementPath`).
pub(crate) fn movement_path_labels(
    mut contexts: EguiContexts,
    movement_path: Res<MovementPath>,
    view: HexMapView,
    game_state: Option<Res<crate::GameStateResource>>,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
) {
    let HexMapView {
        layout,
        overlay,
        game_map,
        cameras,
    } = view;
    if movement_path.legs.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);

    // Determine if the selected unit is a gunboat for direction annotations.
    // For a stack, the first unit stands in for the group (units sharing a
    // hex share terrain, so boat/land is uniform within a stack).
    let is_gunboat = match &*state {
        PickerState::Selected { source, .. } => {
            let Ok((_, placed)) = placed_units.get(*source) else {
                return;
            };
            placed.unit_id.is_some_and(|uid| {
                game_state
                    .as_deref()
                    .and_then(|gs| gs.0.find_unit(uid))
                    .is_some_and(|unit| {
                        matches!(
                            unit.profile.movement,
                            omdurman_rules::UnitMovement::Gunboat(_)
                        )
                    })
            })
        }
        PickerState::SelectedStack(sel) => {
            let Some(&source) = sel.sources.first() else {
                return;
            };
            let Ok((_, placed)) = placed_units.get(source) else {
                return;
            };
            placed.unit_id.is_some_and(|uid| {
                game_state
                    .as_deref()
                    .and_then(|gs| gs.0.find_unit(uid))
                    .is_some_and(|unit| {
                        matches!(
                            unit.profile.movement,
                            omdurman_rules::UnitMovement::Gunboat(_)
                        )
                    })
            })
        }
        _ => return,
    };
    let board = game_state.as_deref().map(|gs| &gs.0.board);

    for &(from, to) in &movement_path.legs {
        let world_pos = hex_world_pos(to, origin, &overlay.params);
        let world_pos_3d = Vec3::new(world_pos.x, 2.0, world_pos.z);

        let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos_3d) else {
            continue;
        };

        let cost_str = game_map
            .hexes
            .get(&to)
            .map(|_| {
                // The same step price the plot and the engine use (§5.11).
                let cost = floor_movement_cost(
                    &game_map,
                    from,
                    to,
                    is_gunboat,
                    game_state.as_deref().map(|gs| &gs.0),
                );
                // For gunboats, annotate upstream (↑) / downstream (↓) direction (§5.24).
                let dir = if is_gunboat
                    && let Some(b) = board
                    && let Some(dir) = b.step_direction(from, to)
                {
                    match dir {
                        omdurman_rules::board::StepDirection::Upstream => "↑",
                        omdurman_rules::board::StepDirection::Downstream => "↓",
                    }
                } else {
                    ""
                };
                format!("{dir}{cost}")
            })
            .unwrap_or_else(|| "?".into());

        egui::Area::new(egui::Id::new(("path_label", to)))
            .fixed_pos(egui::pos2(screen_pos.x - 8.0, screen_pos.y - 16.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                crate::ui::frames::tag(crate::ui::palette::HUD_SCRIM, 4).show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(cost_str)
                            .color(egui::Color32::WHITE)
                            .size(11.0)
                            .strong(),
                    );
                });
            });
    }
}

// -- Movement overlay: light-green hex outlines ---------------------------------

#[derive(Component)]
pub(crate) struct MovementHexRing;

#[derive(Component)]
pub(crate) struct MovementRangeRing;

#[derive(Component)]
pub(crate) struct MovementZocRing;

/// Cache key for the movement overlay: the selection's budget plus enough
/// identity to know a *different* selection (or a rebuild) from the current
/// rings. `remaining` is what the BFS is budgeted against -- a single unit's
/// remaining MP, or a stack's largest per-unit budget (the fastest unit
/// bounds how far the group can be plotted, since slower units simply drop).
#[derive(PartialEq)]
pub(crate) enum MovementOverlayKey {
    Single { source: Entity, remaining: i16 },
    Stack(Vec<(Entity, i16)>),
}

#[allow(clippy::too_many_arguments)]
pub fn movement_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    view: MovementOverlayCtx,
    selection: PickerReadSelection,
    existing: MovementRingQueries,
    peers: crate::peers::Peers,
    mut last_key: Local<Option<MovementOverlayKey>>,
    (generation, mut seen_generation): (Res<OverlayGeneration>, Local<u32>),
) {
    if generation.invalidates(&mut seen_generation) {
        *last_key = None;
    }
    let MovementOverlayCtx {
        game_map,
        game_state,
    } = view;
    let PickerReadSelection {
        state,
        placed_units,
    } = selection;
    let MovementRingQueries {
        existing_green,
        existing_gray,
        existing_zoc,
    } = existing;
    // Movement rings only ever describe a Movement-phase plot. Outside the
    // movement phase the same `PickerState::Selected` drives *fire*
    // allocation (§6.41), and a movement budget over the map would fight the
    // fire target rings; a stale stack selection from a phase change is
    // likewise not an active mover.
    if game_state
        .as_ref()
        .is_some_and(|gs| !matches!(gs.0.phase, omdurman_rules::Phase::Movement))
    {
        let green: Vec<Entity> = existing_green.iter().collect();
        let gray: Vec<Entity> = existing_gray.iter().collect();
        let zoc: Vec<Entity> = existing_zoc.iter().collect();
        crate::ui::despawn_all(&mut commands, &green);
        crate::ui::despawn_all(&mut commands, &gray);
        crate::ui::despawn_all(&mut commands, &zoc);
        *last_key = None;
        return;
    }
    // Rebuild only when the selection/remaining-MP key actually differs from
    // the one we last built for. We key on the *value* rather than on
    // `Res::is_changed()`: the click handler takes `ResMut<PickerState>` every
    // frame but only writes it on click frames, yet a stray mutable deref
    // elsewhere could still flip the change flag and force a needless rebuild.
    //
    // Resolve the key and BFS inputs *before* touching the old rings or the
    // cache: if the representative unit isn't queryable this frame, bail
    // without either -- so the cache never advances to a key whose rings we
    // didn't actually spawn (which is what stranded the overlay after a single
    // frame).
    let Some((start_coord, budget, is_boat, mover_owner, key)) = (match &*state {
        PickerState::Selected {
            source,
            start_coord,
            remaining_mp,
            ..
        } => {
            let Ok((_, placed)) = placed_units.get(*source) else {
                return;
            };
            Some((
                *start_coord,
                *remaining_mp,
                placed.is_boat,
                omdurman_rules::unit_profiles::section_owner(placed.section_name),
                MovementOverlayKey::Single {
                    source: *source,
                    remaining: *remaining_mp,
                },
            ))
        }
        PickerState::SelectedStack(sel) => {
            let Some(&source) = sel.sources.first() else {
                return;
            };
            let Ok((_, placed)) = placed_units.get(source) else {
                return;
            };
            // The group can be plotted as far as its fastest unit: slower
            // units are dropped along the way as their budgets run out.
            let budget = sel.remaining_mp.iter().copied().max().unwrap_or(0);
            Some((
                sel.start_coord,
                budget,
                placed.is_boat,
                omdurman_rules::unit_profiles::section_owner(placed.section_name),
                MovementOverlayKey::Stack(
                    sel.sources
                        .iter()
                        .zip(&sel.remaining_mp)
                        .map(|(&e, &m)| (e, m))
                        .collect(),
                ),
            ))
        }
        _ => None,
    }) else {
        // No selection: clear any leftover rings and reset the cache.
        let green: Vec<Entity> = existing_green.iter().collect();
        let gray: Vec<Entity> = existing_gray.iter().collect();
        let zoc: Vec<Entity> = existing_zoc.iter().collect();
        crate::ui::despawn_all(&mut commands, &green);
        crate::ui::despawn_all(&mut commands, &gray);
        crate::ui::despawn_all(&mut commands, &zoc);
        *last_key = None;
        return;
    };

    // Nothing changed since we last built the overlay: leave the existing
    // rings in place. (Despawning unconditionally above and then bailing here
    // would erase the overlay one frame after spawning it.)
    if last_key.as_ref() == Some(&key) {
        return;
    }

    // Selection or remaining MP changed: rebuild from scratch.
    let mut rings = crate::overlay::ring_batch(
        &mut commands,
        &hex,
        existing_green
            .iter()
            .chain(existing_gray.iter())
            .chain(existing_zoc.iter()),
    );

    // Compute enemy ZOC hexes for this player.
    let my_player = peers
        .local()
        .unwrap_or(omdurman_types::Player::AngloEgyptian);
    let enemy = my_player.opponent();
    let enemy_zoc = game_state
        .as_ref()
        .map(|gs| crate::zoc::compute_enemy_zoc(&gs.0, enemy, my_player))
        .unwrap_or_default();

    // Cheapest-first search from the *planned* current position
    // (start_coord), accumulating step costs. (A plain BFS that marks a hex
    // on first discovery under-reports range with mixed 1/3/+5 costs: an
    // early expensive route hides a later cheap one.) When the path is empty
    // start_coord == placed.coord.
    let gs_state = game_state.as_deref().map(|gs| &gs.0);
    let mut best: HashMap<HexCoord, i16> = HashMap::from([(start_coord, 0)]);
    let mut stops: HashSet<HexCoord> = HashSet::new();
    let mut heap = BinaryHeap::from([Reverse((0i16, start_coord.q, start_coord.r))]);
    while let Some(Reverse((cost_so_far, q, r))) = heap.pop() {
        let cur = HexCoord::new(q, r);
        if best.get(&cur).is_some_and(|&b| b < cost_so_far) {
            continue;
        }
        for neighbor in cur.neighbors() {
            if neighbor == start_coord {
                continue;
            }
            // §5.51: friendly-occupied hexes are enterable and passable (the
            // stacking cap binds only where the move *ends*); only
            // *enemy*-occupied hexes wall off a route (§7.1). §9.346: a
            // Dervish mover may enter/pass the Palace even though GORDON holds
            // it, so the Palace is never an enemy wall.
            let dest_is_palace = game_map.hexes.get(&neighbor).is_some_and(|h| {
                h.name
                    .as_deref()
                    .and_then(omdurman_types::Location::from_tile_name)
                    == Some(omdurman_types::Location::Palace)
            });
            let enemy_occupied = mover_owner.is_some()
                && placed_units.iter().any(|(_, u)| {
                    u.coord == neighbor
                        && omdurman_rules::unit_profiles::section_owner(u.section_name)
                            != mover_owner
                });
            if enemy_occupied && !dest_is_palace {
                continue;
            }
            // 0 = closed: impassable terrain, or a wall / closed Zariba
            // hexside (§5.23; gates and breaches pass).
            let step = floor_movement_cost(&game_map, cur, neighbor, is_boat, gs_state);
            if step <= 0 {
                continue;
            }
            let new_cost = cost_so_far + step;
            if new_cost > budget || best.get(&neighbor).is_some_and(|&b| b <= new_cost) {
                continue;
            }
            best.insert(neighbor, new_cost);
            // §5.41: ZOC hexes are reachable as path termini but the search
            // does not expand from them.
            if enemy_zoc.contains(&neighbor) {
                stops.insert(neighbor);
            } else {
                stops.remove(&neighbor);
                heap.push(Reverse((new_cost, neighbor.q, neighbor.r)));
            }
        }
    }

    let mut green_spawned = 0u32;
    let mut gray_spawned = 0u32;
    let mut zoc_spawned = 0u32;
    for &reached in best.keys().filter(|&&h| h != start_coord) {
        if stops.contains(&reached) {
            // Yellow: a terminus inside an enemy ZOC.
            rings.ring(MovementZocRing, reached, 1.5, 1.0, &hex.assets.yellow);
            zoc_spawned += 1;
        } else if start_coord.neighbors().contains(&reached) {
            rings.ring(MovementHexRing, reached, 1.5, 1.0, &hex.assets.light_green);
            green_spawned += 1;
        } else {
            rings.ring(MovementRangeRing, reached, 1.5, 1.0, &hex.assets.gray);
            gray_spawned += 1;
        }
    }

    info!(
        green_spawned,
        gray_spawned, zoc_spawned, budget, "movement_overlay_mesh: done"
    );
    *last_key = Some(key);
}

// -- Deployment-zone overlay (Setup phase): brown hex outlines ------------------

#[derive(Component)]
pub(crate) struct DeploymentZoneRing;

/// During [`omdurman_rules::Phase::Setup`], outline the hexes where the local
/// player may deploy (§9.2/§9.3), so setup is legible. Highlights the local
/// faction's zone (or, in an unbound session, the active player's). Cleared
/// automatically once play leaves Setup. Rebuilt only when the phase/faction key
/// changes, to avoid per-frame entity churn (cf. `movement_overlay_mesh`).
pub fn deployment_zone_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    game_state: Option<Res<crate::GameStateResource>>,
    peers: crate::peers::Peers,
    existing: Query<Entity, With<DeploymentZoneRing>>,
    mut last_key: Local<Option<omdurman_types::Player>>,
    (generation, mut seen_generation): (Res<OverlayGeneration>, Local<u32>),
) {
    if generation.invalidates(&mut seen_generation) {
        *last_key = None;
    }
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let in_setup = game_state
        .as_deref()
        .is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup));
    let Some(gs) = game_state.as_deref().filter(|_| in_setup) else {
        // Not in setup: clear any leftover rings and reset the cache.
        if last_key.is_some() {
            let existing: Vec<Entity> = existing.iter().collect();
            crate::ui::despawn_all(&mut commands, &existing);
            *last_key = None;
        }
        return;
    };

    // Whose zone to show: the local faction, or the active player in an unbound
    // session (no faction binding).
    let who = peers.local().unwrap_or(gs.0.active_player);
    if *last_key == Some(who) {
        return; // unchanged -- leave the rings in place
    }
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
    *last_key = Some(who);

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    // Iterate the full board terrain (not the clipped game_map) so edge
    // hexes that the overlay doesn't cover still get deployment rings. A hex
    // is highlighted if it is a legal deploy hex for *either* a gunboat or a
    // land unit (§5.22 makes the FoK zones boat/land-exclusive), so the player
    // sees the full set: Nile hexes for the gunboats, and the garrison /
    // landmark / wall-adjacent hexes for the land units.
    for coord in gs.0.board.terrain.keys() {
        let valid = gs.0.in_deployment_zone(who, *coord, true)
            || gs.0.in_deployment_zone(who, *coord, false);
        if valid {
            let pos = hex_world_pos(*coord, origin, &overlay.params);
            // Saturated green: the pale ring read as part of the printed
            // hex grid on the sepia map.
            commands.spawn((
                DeploymentZoneRing,
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.green.clone()),
                Transform::from_xyz(pos.x, 1.4, pos.z).with_scale(Vec3::splat(size)),
                Visibility::Visible,
            ));
        }
    }
}

// -- Selection outline: blue (Anglo-Egyptian) / orange (Dervish) ----------------

#[derive(Component)]
pub(crate) struct SelectionRing;

/// Outline the currently focused unit's hex: blue for Anglo-Egyptian, orange
/// for Dervish. A stack selection outlines every unit in the stack. Driven by
/// `PickerState::Selected { source }` / `SelectedStack` so it tracks
/// click / undo / delete. Rebuilt only when the focused entity set changes.
pub fn selection_outline_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    state: Res<PickerState>,
    placed_units: Query<&PlacedUnit>,
    existing: Query<Entity, With<SelectionRing>>,
    mut last_sources: Local<Option<Vec<Entity>>>,
    (generation, mut seen_generation): (Res<OverlayGeneration>, Local<u32>),
) {
    if generation.invalidates(&mut seen_generation) {
        *last_sources = None;
    }
    let crate::HexRender {
        assets, overlay, ..
    } = hex;
    let sources: Vec<Entity> = match &*state {
        PickerState::Selected { source, .. } => vec![*source],
        PickerState::SelectedStack(sel) => sel.sources.clone(),
        PickerState::SelectedTile(sel) => sel.sources.clone(),
        _ => Vec::new(),
    };
    if *last_sources == Some(sources.clone()) {
        return;
    }
    let old: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &old);
    *last_sources = Some(sources.clone());
    let Some(&first) = sources.first() else {
        return;
    };
    let Ok(placed) = placed_units.get(first) else {
        return;
    };
    let owner = omdurman_rules::unit_profiles::section_owner(placed.section_name);
    let material = match owner {
        Some(omdurman_types::Player::Dervish) => assets.orange.clone(),
        // Anglo-Egyptian, or unknown (editor/unbound): blue.
        _ => assets.blue.clone(),
    };
    let sprite_size = overlay.params.hex_size * SPRITE_HEX_FRACTION;
    let outline_size = sprite_size * 1.18;
    // Spawn the outline as a *child* of each unit entity so it inherits the
    // unit's Transform -- including the per-index stack offset applied by
    // `layout_stacked_units` -- and rides the counter as it moves/animates,
    // rather than sitting at the raw hex centre. Local +Z maps to world +Y
    // under the counter's `rotation_x(-PI/2)`, so local z = -0.02 places the
    // backing just below the counter (world Y), framing it.
    for entity in &sources {
        commands.entity(*entity).with_children(|parent| {
            parent.spawn((
                SelectionRing,
                Mesh3d(assets.unit_square.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(0.0, 0.0, -0.02).with_scale(Vec3::splat(outline_size)),
                Visibility::Visible,
            ));
        });
    }
}

// -- Hover square: bright preview of which unit a click would select ---------

#[derive(Component)]
pub(crate) struct HoverRing;

/// Resolve the specific placed unit under the cursor each frame (the nearest
/// counter in a stack) and publish it as [`crate::HoveredUnit`]. Reuses the
/// click hit-test (`nearest_placed_unit_at`) so hover and click always agree
/// on which unit is targeted. In a bound game only the local faction's units
/// are highlighted (those a click could actually select).
pub(crate) fn update_hovered_unit(
    ground: Res<crate::picking::PointerGroundHit>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    peers: crate::peers::Peers,
    mut hovered: ResMut<crate::HoveredUnit>,
) {
    let Some(hit) = **ground else {
        hovered.0 = None;
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
    let coord = hit_to_hex(hit, origin, &overlay.params);
    let center = hex_world_pos(coord, origin, &overlay.params);
    let spread = 0.34 * overlay.params.hex_size;
    let local = peers.local();
    let target = nearest_placed_unit_at(
        &placed_units,
        coord,
        center,
        hit,
        spread,
        overlay.params.hex_size,
    )
    .filter(|(_, p)| match local {
        Some(local) => omdurman_rules::unit_profiles::section_owner(p.section_name) == Some(local),
        None => true,
    })
    .map(|(e, _)| e);
    hovered.0 = target;
}

/// Bright square on the unit under the cursor, previewing which counter a
/// click would select. Parented to the unit so it tracks stack offsets and
/// motion. Hidden when the hovered unit is the currently selected one (the
/// selection outline already marks it). Rebuilt only when the hovered entity
/// changes.
pub fn hover_outline_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    hovered: Res<crate::HoveredUnit>,
    state: Res<PickerState>,
    existing: Query<Entity, With<HoverRing>>,
    mut last: Local<Option<Entity>>,
    (generation, mut seen_generation): (Res<OverlayGeneration>, Local<u32>),
) {
    if generation.invalidates(&mut seen_generation) {
        *last = None;
    }
    let crate::HexRender {
        assets, overlay, ..
    } = hex;
    let selected: Vec<Entity> = match &*state {
        PickerState::Selected { source, .. } => vec![*source],
        PickerState::SelectedStack(sel) => sel.sources.clone(),
        PickerState::SelectedTile(sel) => sel.sources.clone(),
        _ => Vec::new(),
    };
    // Don't show the hover square on an already-selected unit (a stack
    // selection marks all of them).
    let target = hovered.0.filter(|e| !selected.contains(e));
    if *last == target {
        return;
    }
    let old: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &old);
    *last = target;
    let Some(entity) = target else {
        return;
    };
    let sprite_size = overlay.params.hex_size * SPRITE_HEX_FRACTION;
    let outline_size = sprite_size * 1.18;
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            HoverRing,
            Mesh3d(assets.unit_square.clone()),
            MeshMaterial3d(assets.hover.clone()),
            Transform::from_xyz(0.0, 0.0, -0.02).with_scale(Vec3::splat(outline_size)),
            Visibility::Visible,
        ));
    });
}

/// Draw every unit's movement path this turn as directional arrows (start ->
/// step -> ... -> current hex), so the route each unit took is visible until the
/// turn ends. The path whose start, end, or any crossed hex is under the cursor
/// is drawn bright; all others are drawn dim.
///
/// Rebuilt only when the paths or the hovered hex change (not every frame): the
/// arrow entities otherwise churn, and -- as with the reachable-range overlay --
/// unconditional per-frame despawn/respawn risks a one-frame flash.
pub fn movement_path_arrows(
    mut commands: Commands,
    assets: Res<crate::render::MovementArrowAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    paths: Res<UnitPaths>,
    hovered: Res<crate::HoveredHex>,
    existing: Query<Entity, With<MovementPathArrow>>,
) {
    if !paths.is_changed() && !hovered.is_changed() {
        return;
    }
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;

    for path in paths.0.values() {
        // A path needs at least a start and one step to draw an arrow.
        if path.len() < 2 {
            continue;
        }
        // Bright if the cursor is on any hex of this path (start/end included).
        let hovered_here = hovered.0.is_some_and(|h| path.contains(&h));
        let material = if hovered_here {
            assets.bright.clone()
        } else {
            assets.dim.clone()
        };

        for pair in path.windows(2) {
            let from = hex_world_pos(pair[0], origin, &overlay.params);
            let to = hex_world_pos(pair[1], origin, &overlay.params);
            let delta = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
            let len = delta.length();
            if len < f32::EPSILON {
                continue;
            }
            let dir = delta / len;
            // Shorten slightly at both ends so consecutive arrows read as
            // separate hops and the head doesn't bury under the next counter.
            let inset = size * 0.18;
            let draw_len = (len - inset).max(len * 0.4);
            let tail = from + dir * ((len - draw_len) * 0.5);
            // The unit arrow points along +Z; rotate that onto the heading and
            // scale length (Z) to the segment, width (X) to a fraction of a hex.
            commands.spawn((
                MovementPathArrow,
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(tail.x, 1.45, tail.z)
                    .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                    .with_scale(Vec3::new(size * 0.5, 1.0, draw_len)),
                Visibility::Visible,
            ));
        }
    }
}

// -- Turn-path shadow: translucent hex rings under committed paths ----------

#[derive(Component)]
pub(crate) struct MovementPathShadow;

/// Spawn a translucent hex ring under every hex in each unit's committed
/// movement path, giving players a persistent visual "footprint" of where
/// units moved this turn. Cleared on turn change (via `UnitPaths` reset) and
/// on exit from gameplay overlays. Rebuilt only when `UnitPaths` changes.
pub fn movement_path_shadows(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    paths: Res<UnitPaths>,
    existing: Query<Entity, With<MovementPathShadow>>,
) {
    if !paths.is_changed() {
        return;
    }
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;

    for path in paths.0.values() {
        if path.is_empty() {
            continue;
        }
        for &coord in path {
            let pos = hex_world_pos(coord, origin, &overlay.params);
            commands.spawn((
                MovementPathShadow,
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.path_shadow.clone()),
                Transform::from_xyz(pos.x, 1.38, pos.z).with_scale(Vec3::splat(size)),
                Visibility::Visible,
            ));
        }
    }
}

// -- Stacked-unit layout: offset co-located counters, expand on hover ----------

/// Despawn every gameplay overlay marker. Registered on exit from each map mode
/// so leaving the board (to the lobby, an editor, or any tool) leaves no
/// stranded movement/fire/melee/retreat/trail/entry/preview rings.
type GameplayOverlayEntities<'w, 's> = Query<
    'w,
    's,
    Entity,
    Or<(
        With<MovementHexRing>,
        With<MovementRangeRing>,
        With<MovementPathArrow>,
        With<MovementPathShadow>,
        With<DeploymentZoneRing>,
        With<PreviewHexRing>,
        With<crate::fire::FireTargetRing>,
        With<crate::melee::MeleeTargetRing>,
        With<crate::retreat::RetreatTargetRing>,
        With<crate::fok_entry::FokEntryRing>,
        With<crate::zoc::ZocRing>,
        With<crate::fire::FireDirectionArrow>,
        With<crate::fire_allocation::AllocationArrow>,
        With<crate::melee::MeleeDirectionArrow>,
        With<crate::melee::AdvanceTargetRing>,
    )>,
>;

/// Parented overlay rings (children of unit entities): the selection outline
/// and hover square. Kept out of [`GameplayOverlayEntities`] so that filter
/// stays under Bevy's `Or` arity limit; despawned alongside the standalone
/// overlays on mode exit.
type ParentedOverlayEntities<'w, 's> =
    Query<'w, 's, Entity, Or<(With<SelectionRing>, With<HoverRing>)>>;

pub(crate) fn clear_gameplay_overlays(
    mut commands: Commands,
    rings: GameplayOverlayEntities<'_, '_>,
    parented: ParentedOverlayEntities<'_, '_>,
    overlays: OverlayRingEntities<'_, '_>,
    mut generation: ResMut<OverlayGeneration>,
) {
    let rings: Vec<Entity> = rings
        .iter()
        .chain(parented.iter())
        .chain(overlays.iter())
        .collect();
    crate::ui::despawn_all(&mut commands, &rings);
    // Invalidate every "rebuild only when the key changes" cache: the rings
    // those caches describe are gone, so the next frame in a map view must
    // rebuild them even though the key itself did not change.
    generation.0 = generation.0.wrapping_add(1);
}

/// Bumped by [`clear_gameplay_overlays`]. Overlay systems that cache the key
/// of the rings they last built compare it against their own copy
/// ([`OverlayGeneration::invalidates`]) and drop the cache when it moved, so
/// overlays come back after a round trip through the menu / lobby.
#[derive(Resource, Default)]
pub struct OverlayGeneration(pub u32);

impl OverlayGeneration {
    /// `true` (and records the new generation in `seen`) when the overlays
    /// were cleared since the caller last looked.
    pub fn invalidates(&self, seen: &mut u32) -> bool {
        let moved = *seen != self.0;
        *seen = self.0;
        moved
    }
}

/// Second-chunk overlay entities (the `Or` tuple above is at Bevy's arity
/// limit): LOS rings, spectator combat markers, and the per-frame board
/// markers (acted rings, howitzer bursts, mines/chain, reinforcement entry).
type OverlayRingEntities<'w, 's> = Query<
    'w,
    's,
    Entity,
    Or<(
        With<crate::los::LosRing>,
        With<crate::timeline::SpectatorCombatMarker>,
        With<crate::render::ActedMarker>,
        With<crate::fire::HowitzerImpactMarker>,
        With<crate::river_placement::MineChainMarker>,
        With<crate::reinforce::ReinforceEntryRing>,
    )>,
>;

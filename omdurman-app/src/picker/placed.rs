//! Placed counters on the board: spawning, stacking layout, movement
//! animation, and per-state visual sync.

use super::*;

/// Sprite quad size as a fraction of the hex size, so a placed counter sits
/// just inside its hex.
pub(crate) const SPRITE_HEX_FRACTION: f32 = 1.05;
/// Height above the ground plane at which placed-unit quads are drawn.
pub(crate) const UNIT_HEIGHT: f32 = 1.0;
/// Seconds a unit takes to slide from one hex to an adjacent one.
const MOVE_ANIM_SECS: f32 = 0.3;

/// Per-index visual offset of a counter within its hex stack. Mirrors the
/// rendering in `layout_stacked_units`: a single unit sits at the centre, a
/// stack fans along a short diagonal so each counter peeks out from under the
/// one above. `spread` is already scaled by hex size; `idx` is the unit's
/// position in the stack sorted by entity id.
fn stack_offset(idx: usize, n: usize, spread: f32) -> Vec3 {
    if n <= 1 {
        return Vec3::ZERO;
    }
    // Centre the fan around the hex: index 0..n-1 -> -(n-1)/2 .. +(n-1)/2.
    // The fan's total half-width is clamped to `2 * spread` so large stacks
    // (5-6 counters) compress instead of poking past the hex outline into
    // the neighbouring hex -- a counter visually sitting on a foreign
    // tribe's hex reads as a stacking violation even when the engine state
    // is legal.
    let raw_half = (n as f32 - 1.0) / 2.0;
    let scale = (2.0 / raw_half).min(1.0);
    let k = (idx as f32 - raw_half) * scale;
    Vec3::new(k * spread, 0.0, k * spread * 0.6)
}

/// How far each stack's centre sits from the hex centre when a hex holds
/// *both* stacks: undisrupted counters fan in the hex's top half (board
/// north, `-Z`), disrupted (upside-down) counters in the bottom half
/// (`+Z`), as a fraction of the hex size (§6.22 CRT note: disrupted units
/// are turned upside-down).
const STACK_SPLIT: f32 = 0.16;

/// The `Z` offset of one group's fan centre. Each group is pushed toward its
/// half only when the *other* group is present -- a hex holding just one
/// kind of counter keeps that stack centred.
fn stack_group_z(disrupted: bool, group_n: usize, other_n: usize, hex_size: f32) -> f32 {
    if group_n == 0 || other_n == 0 {
        return 0.0;
    }
    let half = STACK_SPLIT * hex_size;
    if disrupted { half } else { -half }
}

/// Full in-plane offset of one counter in a hex: the group's fan
/// ([`stack_offset`]) plus the two-stack partition offset
/// ([`stack_group_z`], normal top / disrupted bottom). Shared by the
/// layout pass and the hover/click hit-test so the two can never drift.
fn counter_offset(
    disrupted: bool,
    idx: usize,
    group_n: usize,
    other_n: usize,
    spread: f32,
    hex_size: f32,
) -> Vec3 {
    let fan = stack_offset(idx, group_n, spread);
    Vec3::new(
        fan.x,
        0.0,
        fan.z + stack_group_z(disrupted, group_n, other_n, hex_size),
    )
}

/// Among the units occupying `coord`, pick the one whose rendered position
/// (two-stack layout: undisrupted top / disrupted bottom, each fanning via
/// `counter_offset`) is nearest the cursor `hit` point -- so clicking a
/// stacked hex selects the specific counter under the cursor, not always
/// the first in the stack. `center` is the hex's world position; `spread`
/// and `hex_size` should match the rendered layout (use the expanded
/// spread, since a hex is hovered when clicked).
pub(crate) fn nearest_placed_unit_at<'a>(
    units: &'a Query<(Entity, &PlacedUnit)>,
    coord: HexCoord,
    center: Vec3,
    hit: Vec3,
    spread: f32,
    hex_size: f32,
) -> Option<(Entity, &'a PlacedUnit)> {
    // Common case: at most one counter on the hex. A lone counter sits
    // centred (its rendered position always wins), so no stack needs laying
    // out and the Vec + sort below are skipped.
    let mut sole: Option<(Entity, &PlacedUnit)> = None;
    let mut count = 0usize;
    for item in units.iter().filter(|(_, u)| u.coord == coord) {
        count += 1;
        if count > 1 {
            break;
        }
        sole = Some(item);
    }
    if count <= 1 {
        return sole;
    }
    let mut stack: Vec<(Entity, &PlacedUnit)> =
        units.iter().filter(|(_, u)| u.coord == coord).collect();
    stack.sort_by_key(|(e, _)| e.to_bits());
    let n_normal = stack.iter().filter(|(_, u)| !u.disrupted).count();
    let n_disrupted = stack.len() - n_normal;
    let mut best: Option<(usize, f32)> = None;
    let (mut normal_idx, mut disrupted_idx) = (0usize, 0usize);
    for (i, (_, u)) in stack.iter().enumerate() {
        let (idx, group_n, other_n) = if u.disrupted {
            let slot = (disrupted_idx, n_disrupted, n_normal);
            disrupted_idx += 1;
            slot
        } else {
            let slot = (normal_idx, n_normal, n_disrupted);
            normal_idx += 1;
            slot
        };
        let off = counter_offset(u.disrupted, idx, group_n, other_n, spread, hex_size);
        let d = (hit.x - (center.x + off.x)).powi(2) + (hit.z - (center.z + off.z)).powi(2);
        match best {
            Some((_, bd)) if d >= bd => {}
            _ => best = Some((i, d)),
        }
    }
    best.and_then(|(i, _)| stack.get(i).copied())
}

// -- Resources ------------------------------------------------------------------

#[derive(Component)]
pub struct PlacedUnit {
    pub coord: HexCoord,
    pub section_name: SectionName,
    pub col: u32,
    pub row: u32,
    pub is_boat: bool,
    /// The rules-engine unit ID, assigned when the unit is first placed
    /// and the corresponding [`omdurman_rules::UnitPlacement`] is created.
    pub unit_id: Option<UnitId>,
    /// Last-rendered disruption state. A disrupted counter is shown
    /// *inverted* (flipped 180 deg in-plane) and dimmed, mirroring the physical
    /// game where a disrupted counter is turned over (rulebook Combat Results
    /// Table note; §5.41). Kept here so the sync system only re-skins the
    /// counter when its state actually changes.
    pub disrupted: bool,
}

/// The route each unit has taken this turn, keyed by rules-engine `UnitId`, in
/// order (index 0 is the hex the unit started the turn on, the last entry is
/// where it now stands). Populated at the authoritative move-apply point
/// (`game_apply::apply_game_event`) so it captures local, remote, and
/// replayed moves for *both* factions alike -- not just the locally-selected
/// unit. Rendered as directional arrows by [`movement_path_arrows`] and cleared
/// wholesale when the active player changes (end of that player's turn), so the
/// paths persist for the whole turn as a review of what moved where.
#[derive(Resource, Default)]
pub struct UnitPaths(pub std::collections::HashMap<UnitId, Vec<HexCoord>>);

impl UnitPaths {
    /// Record a committed step: start a fresh path at `from` for a unit with no
    /// path yet, then append `to`. `from`/`to` are the unit's pre/post-move
    /// hexes for this step, so a multi-step move accumulates the full route.
    pub fn record_step(&mut self, unit: UnitId, from: HexCoord, to: HexCoord) {
        let path = self.0.entry(unit).or_insert_with(|| vec![from]);
        // Guard against a desync where the stored tail doesn't match this step's
        // origin (e.g. an unobserved teleport): restart the path from `from`.
        if path.last() != Some(&from) {
            path.clear();
            path.push(from);
        }
        path.push(to);
    }
}

/// A counter gliding across the board, one hex step at a time: `from` → `to`
/// is the current step, `rest` the remaining waypoints (world positions).
/// Purely visual -- the counter's [`PlacedUnit::coord`] already holds the
/// engine position; the animation only owns the transform until it finishes.
#[derive(Component)]
pub struct MovementAnimation {
    pub from: Vec3,
    pub to: Vec3,
    pub progress: f32,
    pub rest: std::collections::VecDeque<Vec3>,
}

// -- Shared spawn helper --------------------------------------------------------

/// Spawn the mesh + material for a placed counter and return its entity.
///
/// Used both by the optimistic local placement (click handler) and by
/// [`reconcile_unit_sprites`], so the two can't drift in how a counter is
/// built.
pub fn spawn_placed_unit(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    overlay: &HexOverlay,
    world_pos: Vec3,
    placed: PlacedUnit,
) -> Entity {
    let sprite_size = overlay.params.hex_size * SPRITE_HEX_FRACTION;
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        unlit: true,
        alpha_mode: AlphaMode::Mask(0.1),
        ..default()
    });
    commands
        .spawn((
            placed,
            Mesh3d(meshes.add(Rectangle::new(sprite_size, sprite_size))),
            MeshMaterial3d(material),
            Transform::from_xyz(world_pos.x, UNIT_HEIGHT, world_pos.z)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
            Visibility::Visible,
        ))
        .id()
}

// -- Startup: load sprite handles for the picker -------------------------------

/// Lay out the counters that share a hex so they don't overlap (or z/y-fight):
/// each is nudged by a small per-index offset in xz and a tiny per-index step in
/// y (so the quads never sit in the same plane). When the hovered hex holds more
/// than one counter, that hex's units fan out to ~2x the spread so all of them
/// are readable. Transforms are eased toward the target each frame, giving the
/// expand/collapse a smooth animation; a counter within [`SETTLE_EPSILON`] of
/// its slot snaps there and is left alone (no `Transform` write, so no
/// re-extraction), and while any counter is still easing the frame is marked
/// busy ([`crate::activity::Activity`]). Units currently sliding between hexes
/// (`MovementAnimation`) are left to `animate_unit_movement`.
pub fn layout_stacked_units(
    time: Res<Time>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    hovered: Res<crate::HoveredHex>,
    mut activity: ResMut<crate::activity::Activity>,
    mut units: Query<(Entity, &PlacedUnit, &mut Transform), Without<MovementAnimation>>,
    mut grouping: Local<Option<StackGrouping>>,
) {
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;

    // Group the (non-animating) counters by hex, in a stable order (entity id),
    // so each unit's slot index is deterministic frame to frame. Each group
    // carries its disruption flag: a hex renders *two* stacks, undisrupted
    // counters in the top half and disrupted (upside-down) ones in the
    // bottom half (§6.22 CRT note).
    //
    // The grouping only depends on *which* counters stand where and whether
    // they are disrupted, so it is rebuilt only when that set changes: an
    // idle board (even one mid-animation) reuses the last grouping instead of
    // rebuilding the map and re-sorting every stack every frame. The
    // fingerprint hashes exactly the grouping inputs (entity, hex, flag), so
    // any spawn, despawn, move, or disruption flip is caught.
    let mut fingerprint = std::hash::DefaultHasher::new();
    for (entity, placed, _) in &units {
        entity.to_bits().hash(&mut fingerprint);
        placed.coord.hash(&mut fingerprint);
        placed.disrupted.hash(&mut fingerprint);
    }
    let fingerprint = fingerprint.finish();
    if grouping
        .as_ref()
        .is_none_or(|g| g.fingerprint != fingerprint)
    {
        let mut by_hex: HashMap<HexCoord, Vec<(Entity, bool)>> = HashMap::new();
        for (entity, placed, _) in &units {
            by_hex
                .entry(placed.coord)
                .or_default()
                .push((entity, placed.disrupted));
        }
        for ents in by_hex.values_mut() {
            ents.sort_by_key(|(e, _)| e.to_bits());
        }
        *grouping = Some(StackGrouping {
            fingerprint,
            by_hex,
        });
    }
    let Some(grouping) = grouping.as_ref() else {
        return;
    };
    let by_hex = &grouping.by_hex;

    let lerp = (time.delta_secs() * 12.0).min(1.0);

    for (entity, placed, mut transform) in &mut units {
        let stack = &by_hex[&placed.coord];
        let n_normal = stack.iter().filter(|(_, d)| !*d).count();
        let n_disrupted = stack.len() - n_normal;
        let (n_group, other_n) = if placed.disrupted {
            (n_disrupted, n_normal)
        } else {
            (n_normal, n_disrupted)
        };
        let idx = stack
            .iter()
            .filter(|(_, d)| *d == placed.disrupted)
            .position(|(e, _)| *e == entity)
            .unwrap_or(0);
        let center = hex_world_pos(placed.coord, origin, &overlay.params);

        // Per-group offset: the group's fan (a single unit sits centred in
        // its half; a stack fans along a short diagonal so each counter
        // peeks out from under the one above) plus the top/bottom split.
        // Hovering the *hex* — anywhere on it, not just over a counter
        // symbol — widens the spread for readability: the gate is the
        // board-plane `HoveredHex`, which unit quads cannot block (picking
        // is plane-only) and the hover tooltip cannot steal (it is
        // click-through, see `hover_tooltip`), so the accordion holds steady
        // wherever the cursor sits on the tile instead of flickering.
        // `stack_offset` clamps the fan inside the hex outline (spilling
        // into a neighbouring hex reads as illegal stacking).
        let expanded = stack.len() > 1 && hovered.0 == Some(placed.coord);
        let spread = if expanded { 0.26 } else { 0.14 } * size;
        let off = counter_offset(placed.disrupted, idx, n_group, other_n, spread, size);
        // A tiny per-index height step keeps the quads out of the same
        // plane (no y-fighting); the index is global to the hex so the two
        // overlapping halves still layer deterministically. The hovered
        // stack lifts a hair more.
        let global_idx = stack.iter().position(|(e, _)| *e == entity).unwrap_or(0);
        let y_step = if expanded { 0.12 } else { 0.04 };
        let target = Vec3::new(
            center.x + off.x,
            UNIT_HEIGHT + global_idx as f32 * y_step,
            center.z + off.z,
        );
        // Read through the `Mut` without touching it: only a counter that
        // actually moves gets a changed `Transform`.
        let current = transform.translation;
        if current == target {
            continue;
        }
        if current.distance(target) <= SETTLE_EPSILON {
            transform.translation = target;
        } else {
            transform.translation = current.lerp(target, lerp);
            activity.keep_running();
        }
    }
}

/// The cached stack grouping of [`layout_stacked_units`]: a fingerprint of
/// its inputs (entity, hex, disrupted) plus the grouping those inputs
/// produce. The fingerprint makes any change to the inputs detectable
/// without rebuilding the grouping. The type leaks through the system's
/// `Local` parameter, hence the visibility.
pub(crate) struct StackGrouping {
    fingerprint: u64,
    by_hex: std::collections::HashMap<HexCoord, Vec<(Entity, bool)>>,
}
/// How close (world units) an easing counter must come to its stack slot
/// before it snaps there and stops being updated.
pub const SETTLE_EPSILON: f32 = 1e-3;

// -- Animation: lerp unit movement ----------------------------------------------

pub fn animate_unit_movement(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MovementAnimation)>,
    mut commands: Commands,
    mut activity: ResMut<crate::activity::Activity>,
) {
    for (entity, mut transform, mut anim) in query.iter_mut() {
        activity.keep_running();
        anim.progress += time.delta_secs() / MOVE_ANIM_SECS;
        if anim.progress >= 1.0 {
            transform.translation = anim.to;
            if let Some(next) = anim.rest.pop_front() {
                // Next hex of the route.
                anim.from = anim.to;
                anim.to = next;
                anim.progress = 0.0;
            } else {
                commands.entity(entity).remove::<MovementAnimation>();
            }
        } else {
            let t = anim.progress;
            let ease = smoothstep(t);
            transform.translation = anim.from.lerp(anim.to, ease);
        }
    }
}

/// Smoothstep easing: 0 at t=0, 1 at t=1, zero slope at both ends.
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

// -- Disruption visuals: inverted + dimmed counter ------------------------------

/// Lay a counter quad flat on the ground, optionally *inverted* (turned over)
/// to show disruption. Inversion is a 180 deg spin about the vertical axis, the
/// 3D analogue of flipping the physical counter face-down (rulebook Combat
/// Results Table note; §5.41).
fn counter_rotation(disrupted: bool) -> Quat {
    let flat = Quat::from_rotation_x(-std::f32::consts::PI / 2.0);
    if disrupted {
        Quat::from_rotation_y(std::f32::consts::PI) * flat
    } else {
        flat
    }
}

/// Marker on a counter the local player just placed, before the
/// host-sequenced echo reaches the engine. [`reconcile_unit_sprites`] binds
/// it to its rules unit once the engine has it, and drops it (returning the
/// counter to the picker) once the submission is confirmed but the engine
/// rejected it. `frames` is a short grace period covering the gap between the
/// click spawning the sprite and the submission reaching
/// `PendingEdits::unconfirmed`.
#[derive(Component, Default)]
pub struct PendingPlacement {
    frames: u8,
}

/// Frames an optimistic counter survives without an in-flight submission.
const PENDING_PLACEMENT_GRACE_FRAMES: u8 = 3;

/// Longest route (in hexes) glided hex by hex; anything longer snaps.
const MAX_GLIDE_STEPS: usize = 16;

/// A counter-sheet cell: `(section, col, row)`.
type SpriteKey = (SectionName, u32, u32);

/// Per-counter data reconciled by [`reconcile_unit_sprites`].
type SpriteReconcileItem = (
    Entity,
    &'static mut PlacedUnit,
    &'static mut Transform,
    &'static MeshMaterial3d<StandardMaterial>,
    &'static mut Visibility,
    Option<&'static mut PendingPlacement>,
);

/// Resources read/written by [`reconcile_unit_sprites`], bundled to stay
/// under Bevy's system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct SpriteReconcileCtx<'w> {
    game_state: Res<'w, crate::GameStateResource>,
    board: crate::BoardGeometry<'w>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    asset_server: Res<'w, AssetServer>,
    pending: Res<'w, crate::PendingEdits>,
    paths: Res<'w, UnitPaths>,
    mode: Res<'w, State<crate::AppMode>>,
    picker: ResMut<'w, UnitPicker>,
    picker_state: ResMut<'w, PickerState>,
    activity: ResMut<'w, crate::activity::Activity>,
}

/// The board's counters are a *projection* of the rules engine state: this
/// system reconciles the sprite world to `GameState.units` every frame, in
/// every app state (live play, replay, spectating):
///
/// * binds an optimistic local placement ([`PendingPlacement`]) to its rules
///   unit once the engine has it, and drops it once the engine rejected it;
/// * despawns sprites whose unit left the engine (eliminated by fire/melee
///   §6/§7, desertion §8.2, GORDON's fall §9.346, a setup pickup §9.2/§9.3, or
///   a rebuild to an earlier timeline position);
/// * moves displaced sprites -- gliding along the route recorded in
///   [`UnitPaths`] (or a single adjacent hop), snapping otherwise so a scrub
///   jump never slides a counter straight across walls and foreign stacks;
///   a "Friendlies" counter loaded on a gunboat (§5.21) rides at the boat's
///   hex;
/// * mirrors disruption: a disrupted counter is shown inverted and dimmed
///   (rulebook Combat Results Table note; §5.41);
/// * spawns sprites for engine units that have none (remote placements, AI
///   deployments, replayed records -- texture from the picker's
///   `sprites/{section}_{col}_{row}.webp` naming);
/// * keeps the picker tray equal to "every counter not on the board".
///
/// Counters are hidden outside the Game view (menu / lobby), so returning
/// to the board needs no snapshot: the sprites are re-derived here.
///
/// It runs only when one of its inputs moved -- the engine state, the
/// recorded routes, the app mode, the picker (its sprite handles load
/// late), counters spawned or despawned elsewhere -- or while an optimistic
/// placement awaits its echo (that keeps the frames coming, see
/// [`crate::activity::Activity`]); an idle board costs nothing.
pub fn reconcile_unit_sprites(
    mut commands: Commands,
    ctx: SpriteReconcileCtx,
    mut query: Query<SpriteReconcileItem>,
    mut removed: RemovedComponents<PlacedUnit>,
) {
    let SpriteReconcileCtx {
        game_state,
        board: crate::BoardGeometry { layout, overlay },
        mut meshes,
        mut materials,
        asset_server,
        pending,
        paths,
        mode,
        mut picker,
        mut picker_state,
        mut activity,
    } = ctx;
    // One look over the counters (`iter_mut` without writing marks nothing
    // changed): any spawned since the last run, any awaiting its echo.
    let (mut fresh, mut awaiting_echo) = (false, false);
    for (_, placed, ..) in query.iter_mut() {
        fresh |= placed.is_added();
        awaiting_echo |= placed.unit_id.is_none();
    }
    let despawned = removed.read().count() > 0;
    if !(game_state.is_changed()
        || paths.is_changed()
        || mode.is_changed()
        || picker.is_changed()
        || fresh
        || despawned
        || awaiting_echo)
    {
        return;
    }
    if awaiting_echo {
        // The grace period counts frames: keep them coming.
        activity.keep_running();
    }
    let gs = &game_state.0;
    // One lookup table per run (`find_unit` is a linear scan).
    let by_id: std::collections::HashMap<UnitId, &omdurman_rules::UnitPlacement> =
        gs.units.iter().map(|u| (u.id, u)).collect();
    let origin = layout.adjusted_origin(&overlay.params);
    let world = |hex: HexCoord| {
        let p = hex_world_pos(hex, origin, &overlay.params);
        Vec3::new(p.x, UNIT_HEIGHT, p.z)
    };
    let shown = if **mode == crate::AppMode::Game {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    let mut seen: std::collections::HashSet<UnitId> = std::collections::HashSet::new();
    let mut on_board: std::collections::HashSet<SpriteKey> = std::collections::HashSet::new();

    for (entity, mut placed, mut transform, material, mut visibility, pending_placement) in
        query.iter_mut()
    {
        let key = (placed.section_name, placed.col, placed.row);
        let uid = match placed.unit_id {
            Some(uid) => uid,
            None => {
                // Optimistic local placement awaiting its echo.
                let resolved = unit_id_for_section_pos(key.0, key.1 as u8, key.2 as u8)
                    .filter(|id| by_id.contains_key(id) && !seen.contains(id));
                if let Some(id) = resolved {
                    placed.unit_id = Some(id);
                    commands.entity(entity).remove::<PendingPlacement>();
                    id
                } else {
                    let in_flight = pending.unconfirmed.iter().any(|(_, ev)| {
                        matches!(ev, GameEvent::PlaceUnit { sprite, .. }
                            if sprite.section_name == key.0
                                && sprite.col == key.1
                                && sprite.row == key.2)
                    });
                    let young = pending_placement
                        .as_ref()
                        .is_some_and(|p| p.frames < PENDING_PLACEMENT_GRACE_FRAMES);
                    if in_flight || young {
                        if let Some(mut p) = pending_placement {
                            p.frames = p.frames.saturating_add(1);
                        }
                        on_board.insert(key);
                        if *visibility != shown {
                            *visibility = shown;
                        }
                    } else {
                        // Echoed but rejected by the engine (or never
                        // submitted): back to the tray.
                        commands.entity(entity).despawn();
                    }
                    continue;
                }
            }
        };
        let Some(unit) = by_id.get(&uid).copied().filter(|_| !seen.contains(&uid)) else {
            // Gone from the engine (or a duplicate sprite): drop it.
            commands.entity(entity).despawn();
            continue;
        };
        seen.insert(uid);
        on_board.insert(key);

        // (§5.21: a loaded counter stands on its gunboat's hex.)
        let hex = unit.position;
        if hex != placed.coord {
            let route: Option<Vec<HexCoord>> = paths
                .0
                .get(&uid)
                .filter(|p| p.last() == Some(&hex))
                .and_then(|p| {
                    let start = p.iter().rposition(|h| *h == placed.coord)?;
                    Some(p[start + 1..].to_vec())
                })
                .filter(|steps| !steps.is_empty() && steps.len() <= MAX_GLIDE_STEPS)
                .or_else(|| (placed.coord.distance(hex) == 1).then(|| vec![hex]));
            match route {
                Some(steps) => {
                    // Glide along the real route. Inserting over an in-flight
                    // animation restarts it from the current position.
                    let mut rest: std::collections::VecDeque<Vec3> =
                        steps.into_iter().map(world).collect();
                    let to = rest.pop_front().unwrap_or_else(|| world(hex));
                    commands.entity(entity).try_insert(MovementAnimation {
                        from: transform.translation,
                        to,
                        progress: 0.0,
                        rest,
                    });
                }
                None => {
                    // Teleport (scrub jump, resync): snap to the destination.
                    transform.translation = world(hex);
                    commands.entity(entity).remove::<MovementAnimation>();
                }
            }
            placed.coord = hex;
        }

        let disrupted = unit.state.disrupted;
        if disrupted != placed.disrupted {
            placed.disrupted = disrupted;
            transform.rotation = counter_rotation(disrupted);
            if let Some(mut mat) = materials.get_mut(&material.0) {
                mat.base_color = if disrupted {
                    crate::render::overlay_palette::COUNTER_DISRUPTED
                } else {
                    Color::WHITE
                };
            }
        }
        if *visibility != shown {
            *visibility = shown;
        }
    }

    // Spawn a sprite for every engine unit that has none yet.
    for unit in &gs.units {
        if seen.contains(&unit.id) {
            continue;
        }
        let (section, col, row) = unit.id.section_pos();
        let (col, row) = (u32::from(col), u32::from(row));
        on_board.insert((section, col, row));
        let handle = picker
            .all
            .iter()
            .find(|(sn, c, r, _, _)| *sn == section && *c == col && *r == row)
            .map(|(_, _, _, h, _)| h.clone())
            .unwrap_or_else(|| {
                asset_server.load(super::sprite_asset_path(&format!("{section}_{col}_{row}")))
            });
        let hex = unit.position;
        let entity = spawn_placed_unit(
            &mut commands,
            &mut meshes,
            &mut materials,
            handle,
            &overlay,
            world(hex),
            PlacedUnit {
                coord: hex,
                section_name: section,
                col,
                row,
                is_boat: unit.profile.kind.is_boat(),
                unit_id: Some(unit.id),
                // Re-skinned next frame if the unit is disrupted.
                disrupted: false,
            },
        );
        commands.entity(entity).insert(shown);
    }

    sync_picker_tray(&mut picker, &mut picker_state, &on_board);
}

/// Keep the picker tray equal to "every counter not on the board": drop
/// counters that are (engine units + optimistic placements) and return the
/// ones that left the board (setup pickup, rejected placement, rebuild), in
/// counter-sheet order. Adjusts an in-progress `Placing` index so it keeps
/// pointing at the same counter. Touches the resources only when something
/// actually changes.
fn sync_picker_tray(
    picker: &mut ResMut<UnitPicker>,
    picker_state: &mut ResMut<PickerState>,
    on_board: &std::collections::HashSet<SpriteKey>,
) {
    let key = |u: &PickerUnit| (u.section_name, u.col, u.row);
    let in_tray: std::collections::HashSet<SpriteKey> = picker.available.iter().map(key).collect();
    let stale = in_tray.iter().any(|k| on_board.contains(k));
    let missing: Vec<usize> = picker
        .all
        .iter()
        .enumerate()
        .filter(|(_, (sn, c, r, _, _))| {
            let k = (*sn, *c, *r);
            !on_board.contains(&k) && !in_tray.contains(&k)
        })
        .map(|(i, _)| i)
        .collect();
    if !stale && missing.is_empty() {
        return;
    }

    // Removals.
    let mut i = 0;
    while i < picker.available.len() {
        if on_board.contains(&key(&picker.available[i])) {
            picker.available.remove(i);
            match placing_index(picker_state) {
                Some(idx) if idx == i => **picker_state = PickerState::Idle,
                Some(idx) if idx > i => set_placing_index(picker_state, idx - 1),
                _ => {}
            }
        } else {
            i += 1;
        }
    }

    // Returns, inserted where they sit in the counter-sheet order.
    let sheet_order: std::collections::HashMap<SpriteKey, usize> = picker
        .all
        .iter()
        .enumerate()
        .map(|(i, (sn, c, r, _, _))| ((*sn, *c, *r), i))
        .collect();
    for all_idx in missing {
        let (section_name, col, row, handle, _) = picker.all[all_idx].clone();
        let at = picker
            .available
            .iter()
            .position(|u| sheet_order.get(&key(u)).is_some_and(|&o| o > all_idx))
            .unwrap_or(picker.available.len());
        // Boat-ness from the sprite profile (the engine's source of truth).
        let is_boat = unit_id_for_section_pos(section_name, col as u8, row as u8)
            .and_then(omdurman_rules::unit_profiles::profile_for_unit)
            .is_some_and(|p| p.kind.is_boat());
        picker.available.insert(
            at,
            PickerUnit {
                section_name,
                col,
                row,
                handle,
                is_boat,
                visible: true,
                offered: true,
                egui_texture: None,
                annotations_loaded: false,
            },
        );
        if let Some(idx) = placing_index(picker_state)
            && idx >= at
        {
            set_placing_index(picker_state, idx + 1);
        }
    }
}

/// The tray index an in-progress placement points at (read-only, so it does
/// not trip change detection).
fn placing_index(state: &ResMut<PickerState>) -> Option<usize> {
    match &**state {
        PickerState::Placing { unit_idx, .. } => Some(*unit_idx),
        _ => None,
    }
}

fn set_placing_index(state: &mut ResMut<PickerState>, idx: usize) {
    if let PickerState::Placing { unit_idx, .. } = &mut **state {
        *unit_idx = idx;
    }
}

// -- Cancel placement/movement on right-click ----------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Idle frames cost nothing: a settled stack is not rewritten ---------

    #[derive(Resource, Default)]
    struct ChangedTransforms(usize);

    fn count_changed(q: Query<(), Changed<Transform>>, mut n: ResMut<ChangedTransforms>) {
        n.0 += q.iter().count();
    }

    /// A counter eases into its stack slot while the frame is marked busy,
    /// then snaps and is left alone: no `Transform` write (so nothing
    /// downstream re-runs) and no request for more frames.
    #[test]
    fn a_settled_counter_is_not_rewritten_and_asks_for_no_frames() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(10),
            ))
            .insert_resource(HexOverlay::default())
            .insert_resource(omdurman_board_ui::board_store::default_layout())
            .init_resource::<crate::HoveredHex>()
            .init_resource::<crate::activity::Activity>()
            .init_resource::<ChangedTransforms>()
            .add_systems(Update, (layout_stacked_units, count_changed).chain());
        app.world_mut().spawn((
            PlacedUnit {
                coord: HexCoord::new(3, 3),
                section_name: SectionName::Taiasha,
                col: 0,
                row: 0,
                is_boat: false,
                unit_id: None,
                disrupted: false,
            },
            Transform::from_xyz(500.0, 0.0, 500.0),
        ));

        // Easing in: busy frames.
        app.update();
        app.update();
        assert!(
            app.world()
                .resource::<crate::activity::Activity>()
                .is_busy()
        );

        // Let it arrive.
        for _ in 0..400 {
            app.update();
        }
        // (No `request_redraws` here to consume the flag: clear it by hand.)
        *app.world_mut().resource_mut::<crate::activity::Activity>() = Default::default();
        app.world_mut().resource_mut::<ChangedTransforms>().0 = 0;

        for _ in 0..5 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<ChangedTransforms>().0,
            0,
            "a resting counter's Transform is not touched"
        );
        assert!(
            !app.world()
                .resource::<crate::activity::Activity>()
                .is_busy(),
            "a resting board asks for no frames"
        );
    }

    // -- Two-stack layout: undisrupted top / disrupted bottom (§6.22) ------

    const SIZE: f32 = 1.0;

    /// A hex holding both stacks separates them: normal counters sit in the
    /// top half (negative Z, board north), disrupted ones in the bottom
    /// half, each by `STACK_SPLIT` of the hex size.
    #[test]
    fn mixed_hex_splits_normal_top_disrupted_bottom() {
        let normal = counter_offset(false, 0, 1, 1, 0.0, SIZE);
        let disrupted = counter_offset(true, 0, 1, 1, 0.0, SIZE);
        assert!(normal.z < 0.0 && disrupted.z > 0.0);
        assert_eq!(
            normal.z.abs(),
            disrupted.z.abs(),
            "the two halves are symmetric about the hex centre"
        );
        assert_eq!(normal.z.abs(), STACK_SPLIT * SIZE);
        // With a single counter per group there is no fan: the split is the
        // whole offset.
        assert_eq!(normal.x, 0.0);
        assert_eq!(disrupted.x, 0.0);
    }

    /// A hex holding only one kind of counter keeps it centred — the split
    /// only applies when both stacks are present: the sole group's offset
    /// is exactly the plain fan.
    #[test]
    fn sole_group_stays_centred() {
        for idx in 0..3usize {
            let only_normal = counter_offset(false, idx, 3, 0, 0.1, SIZE);
            let only_disrupted = counter_offset(true, idx, 3, 0, 0.1, SIZE);
            let plain = stack_offset(idx, 3, 0.1);
            assert_eq!(only_normal.z, plain.z);
            assert_eq!(only_disrupted.z, plain.z);
            assert_eq!(only_normal.x, plain.x);
            assert_eq!(only_disrupted.x, plain.x);
        }
    }

    /// The split offset rides on top of the group's fan: the fan geometry
    /// (same group, same index, same spread) is identical with and without
    /// the other stack present, except for the `Z` shift.
    #[test]
    fn split_does_not_distort_the_fan() {
        let fan_alone = counter_offset(false, 1, 3, 0, 0.14, SIZE);
        let fan_split = counter_offset(false, 1, 3, 2, 0.14, SIZE);
        assert_eq!(fan_alone.x, fan_split.x);
        assert!(
            (fan_alone.z - fan_split.z).abs() - STACK_SPLIT * SIZE < 1e-6,
            "split only shifts Z by STACK_SPLIT"
        );
    }

    /// Group indices are per group: with equal-sized groups, counter `i` of
    /// the normal stack and counter `i` of the disrupted stack share the
    /// same fan slot (they differ only by the split) — the disrupted fan is
    /// not offset by the normal stack's slots.
    #[test]
    fn group_fans_are_independent() {
        for idx in 0..2usize {
            let normal = counter_offset(false, idx, 2, 2, 0.14, SIZE);
            let disrupted = counter_offset(true, idx, 2, 2, 0.14, SIZE);
            assert_eq!(normal.x, disrupted.x);
            assert_eq!(normal.x, stack_offset(idx, 2, 0.14).x);
        }
    }
}

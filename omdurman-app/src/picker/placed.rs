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

/// Snapshot helpers for mode-transition state saving.
impl PlacedUnit {
    /// Convert this entity's data into a serializable [`PlacedUnitData`].
    pub fn to_data(&self) -> crate::PlacedUnitData {
        crate::PlacedUnitData {
            section_name: self.section_name,
            col: self.col,
            row: self.row,
            coord: self.coord,
            unit_id: self.unit_id,
            disrupted: self.disrupted,
            is_boat: self.is_boat,
        }
    }
}

/// Collect all placed units into snapshot data.
pub fn collect_placed_units(query: &Query<&PlacedUnit>) -> Vec<crate::PlacedUnitData> {
    query.iter().map(|p| p.to_data()).collect()
}

/// The route each unit has taken this turn, keyed by rules-engine `UnitId`, in
/// order (index 0 is the hex the unit started the turn on, the last entry is
/// where it now stands). Populated at the authoritative move-apply point
/// ([`crate::apply_pending_placement`]) so it captures local, remote, and
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

#[derive(Component)]
pub struct MovementAnimation {
    pub from: Vec3,
    pub to: Vec3,
    pub progress: f32,
    pub target_coord: HexCoord,
}

// -- Shared spawn helper --------------------------------------------------------

/// Spawn the mesh + material for a placed counter and return its entity.
///
/// Used both by interactive placement here and by `apply_pending_placement`
/// in `main.rs` when applying inbound/replayed `PlaceUnit` events, so the two
/// paths can't drift in how a counter is built.
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
/// expand/collapse a smooth animation. Units currently sliding between hexes
/// (`MovementAnimation`) are left to `animate_unit_movement`.
pub fn layout_stacked_units(
    time: Res<Time>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    hovered: Res<crate::HoveredHex>,
    mut units: Query<(Entity, &PlacedUnit, &mut Transform), Without<MovementAnimation>>,
) {
    use std::collections::HashMap;

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;

    // Group the (non-animating) counters by hex, in a stable order (entity id),
    // so each unit's slot index is deterministic frame to frame. Each group
    // carries its disruption flag: a hex renders *two* stacks, undisrupted
    // counters in the top half and disrupted (upside-down) ones in the
    // bottom half (§6.22 CRT note).
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
        transform.translation = transform.translation.lerp(target, lerp);
    }
}

// -- Animation: lerp unit movement ----------------------------------------------

pub fn animate_unit_movement(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut MovementAnimation,
        &mut PlacedUnit,
    )>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut anim, mut placed) in query.iter_mut() {
        anim.progress += time.delta_secs() / MOVE_ANIM_SECS;
        if anim.progress >= 1.0 {
            transform.translation = anim.to;
            placed.coord = anim.target_coord;
            info!(
                entity = entity.to_bits(),
                coord.q = placed.coord.q,
                coord.r = placed.coord.r,
                "movement animation complete"
            );
            commands.entity(entity).remove::<MovementAnimation>();
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

/// Mirror each placed counter's disruption state from the authoritative
/// rules engine into its visuals: a disrupted unit is shown inverted and
/// dimmed, recovering to upright/full-colour when the rules engine clears the
/// flag at end of the owning player's turn (rulebook §5.41, Combat Results
/// Table note). Only re-skins a counter when its state actually changes.
pub fn sync_disrupted_visuals(
    game_state: Option<Res<crate::GameStateResource>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(
        &mut PlacedUnit,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let Some(game_state) = game_state else {
        return;
    };
    for (mut placed, mut transform, material) in query.iter_mut() {
        let Some(uid) = placed.unit_id else {
            continue;
        };
        let disrupted = game_state
            .0
            .find_unit(uid)
            .is_some_and(|u| u.state.disrupted);
        if disrupted == placed.disrupted {
            continue;
        }
        placed.disrupted = disrupted;
        transform.rotation = counter_rotation(disrupted);
        if let Some(mut mat) = materials.get_mut(&material.0) {
            // Dim disrupted counters; full brightness when recovered.
            mat.base_color = if disrupted {
                Color::srgb(0.55, 0.55, 0.55)
            } else {
                Color::WHITE
            };
        }
    }
}

/// A "Friendlies" counter loaded onto a gunboat (§5.21) rides *with the
/// boat*: the engine keeps the loaded unit's `position` at its shore hex
/// (`Load` only sets `loaded_on`), so the board would misrepresent the
/// state. This reconcile pins the counter's `coord` to the carrying
/// gunboat's hex for the whole Load→Cross→Disembark mission, and restores
/// the engine position once unloaded. Optimistic counters (no `unit_id`
/// yet) are left alone.
pub fn sync_loaded_units(
    game_state: Option<Res<crate::GameStateResource>>,
    mut query: Query<&mut PlacedUnit>,
) {
    let Some(game_state) = game_state else {
        return;
    };
    for mut placed in query.iter_mut() {
        let Some(uid) = placed.unit_id else {
            continue;
        };
        let Some(unit) = game_state.0.find_unit(uid) else {
            continue;
        };
        let carried_hex = unit
            .state
            .loaded_on
            .and_then(|boat| game_state.0.find_unit(boat))
            .map(|b| b.position)
            .unwrap_or(unit.position);
        if placed.coord != carried_hex {
            placed.coord = carried_hex;
        }
    }
}

/// Despawn the sprite of any counter the rules engine has eliminated. A placed/// counter that carries a rules `UnitId` no longer present in `GameState.units`
/// has been removed by combat (fire/melee, §6/§7), desertion (§8.2), or GORDON's
/// fall (§9.346); its sprite must leave the board too. Counters not yet bound to
/// a rules id (mid-placement) are left alone.
pub fn sync_eliminated_visuals(
    game_state: Option<Res<crate::GameStateResource>>,
    mut commands: Commands,
    query: Query<(Entity, &PlacedUnit)>,
) {
    let Some(game_state) = game_state else {
        return;
    };
    for (entity, placed) in query.iter() {
        let Some(uid) = placed.unit_id else {
            continue;
        };
        if game_state.0.find_unit(uid).is_none() {
            commands.entity(entity).despawn();
        }
    }
}

/// Spectator-only mirror of the rules engine's units onto the board.
///
/// Records whose traces are pure `GameEvent::Effect`s (bot playthroughs,
/// headless runs) carry no `PlaceUnit`/`MoveUnit` visual events, so replaying
/// them rebuilds the engine `GameState` but leaves the board without counters.
/// While [`AppState::Spectating`] is active this system reconciles the sprite
/// world to the scrubbed engine state every frame: spawns sprites for units
/// without one (texture path derived from `UnitId::section_pos`, matching the
/// picker's `sprites/{section}_{col}_{row}.webp` naming), moves displaced
/// sprites, and despawns eliminated ones. It never runs in live play, where
/// the visual events are the source of truth.
pub fn sync_spectator_units(
    mut commands: Commands,
    game_state: Option<Res<crate::GameStateResource>>,
    board: crate::BoardGeometry,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut query: Query<(Entity, &mut PlacedUnit, &mut Transform)>,
) {
    use omdurman_rules::effects::GameState;

    let crate::BoardGeometry { layout, overlay } = board;
    let Some(game_state) = game_state else {
        return;
    };
    let GameState { units, .. } = &game_state.0;
    let origin = layout.adjusted_origin(&overlay.params);

    // Move or despawn existing sprites to match the engine state.
    let mut seen: Vec<UnitId> = Vec::new();
    for (entity, mut placed, mut transform) in query.iter_mut() {
        let Some(uid) = placed.unit_id else {
            continue;
        };
        let Some(unit) = units.iter().find(|u| u.id == uid) else {
            // Eliminated since the last scrub position (or pre-existing sprite
            // from a previous record): drop it.
            commands.entity(entity).despawn();
            continue;
        };
        seen.push(uid);
        if unit.position != placed.coord {
            // Re-sync with the engine state. Only a *single-hex step* is
            // animated: it is a real move along a legal path. Anything
            // farther (a scrub jump across many events, a re-sync after
            // replay bootstrap) is a teleport we must NOT draw as a glide,
            // or counters slice straight across the board -- through
            // walls and foreign stacks, reading as rule violations.
            let to = hex_world_pos(unit.position, origin, &overlay.params);
            if placed.coord.distance(unit.position) == 1 {
                // Glide to the adjacent hex: reuse the live-play movement
                // animation (smoothstep over MOVE_ANIM_SECS). Inserting
                // over an in-flight animation restarts it from the current
                // mid-flight position. `placed.coord` updates immediately
                // so stacking sees the destination; the transform belongs
                // to the animation until it completes (then
                // `layout_stacked_units` lerps the counter into its slot).
                commands.entity(entity).insert(MovementAnimation {
                    from: transform.translation,
                    to: Vec3::new(to.x, UNIT_HEIGHT, to.z),
                    progress: 0.0,
                    target_coord: unit.position,
                });
            } else {
                // Teleport: snap the transform to the destination hex.
                transform.translation = Vec3::new(to.x, UNIT_HEIGHT, to.z);
                commands.entity(entity).remove::<MovementAnimation>();
            }
            placed.coord = unit.position;
        }
        let disrupted = unit.state.disrupted;
        if disrupted != placed.disrupted {
            placed.disrupted = disrupted;
            transform.rotation = counter_rotation(disrupted);
        }
    }

    // Spawn a sprite for every engine unit that has none yet.
    for unit in units {
        if seen.contains(&unit.id) {
            continue;
        }
        let (section, col, row) = unit.id.section_pos();
        let is_boat = matches!(
            unit.profile.movement,
            omdurman_rules::UnitMovement::Gunboat(_)
        );
        let handle = asset_server.load(format!("sprites/{section}_{col}_{row}.webp"));
        let pos = hex_world_pos(unit.position, origin, &overlay.params);
        spawn_placed_unit(
            &mut commands,
            &mut meshes,
            &mut materials,
            handle,
            &overlay,
            pos,
            PlacedUnit {
                coord: unit.position,
                section_name: section,
                col: col as u32,
                row: row as u32,
                is_boat,
                unit_id: Some(unit.id),
                disrupted: unit.state.disrupted,
            },
        );
    }
}

// -- Cancel placement/movement on right-click ----------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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

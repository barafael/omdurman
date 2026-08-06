//! Unit picker: the left sidebar of available counters, placement preview,
//! click handling for place / select / move, and movement animation.
//!
//! Placement and movement are *requested* here by broadcasting
//! [`GameEvent::PlaceUnit`] / [`GameEvent::MoveUnit`]; the authoritative state
//! change (allocating a rules-engine `UnitId`, validating the move against the
//! unit's movement allowance, updating position) happens in
//! `apply_pending_placement`, which consumes those events. Keeping the request
//! and the application separate is what lets the same code path serve live
//! play and history replay.

use bevy::app::Plugin;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::{GameMap, HexLayout};
use omdurman_types::{HexCoord, HexsideRef, Scenario, Terrain};

use std::collections::{HashSet, VecDeque};

use crate::AppState;
use crate::browser::{SpriteAnnotationsResource, section_order};
use crate::camera::RtsCamera;
use crate::events;
use crate::render::{HexOverlay, HexRingAssets};
use crate::util::raycast_ground;
use omdurman_hexmap::{hex_world_pos, hit_to_hex};
use omdurman_net::GameEvent;
use omdurman_rules::{
    MovementPoints, UnitId, UnitPlacement, UnitState, unit_id_for_section_pos,
};

/// The selected unit's rules `UnitId` and hex, if it is engine-tracked.
///
/// Single-unit selections only: a stack selection (movement group) is not a
/// combat/action target, so it reports `None` -- fire, melee, retreat and the
/// action panel all key off a single selected counter.
pub fn selected_unit_id(
    state: &PickerState,
    placed_units: &Query<(Entity, &PlacedUnit)>,
) -> Option<(UnitId, HexCoord)> {
    let PickerState::Selected { source, .. } = state else {
        return None;
    };
    let (_, placed) = placed_units.get(*source).ok()?;
    Some((placed.unit_id?, placed.coord))
}

use omdurman_types::SectionName;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/sprites.rs"));
}

/// Sprite quad size as a fraction of the hex size, so a placed counter sits
/// just inside its hex.
const SPRITE_HEX_FRACTION: f32 = 1.05;
/// Height above the ground plane at which placed-unit quads are drawn.
const UNIT_HEIGHT: f32 = 1.0;
/// Seconds a unit takes to slide from one hex to an adjacent one.
const MOVE_ANIM_SECS: f32 = 0.3;

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
fn coord_passable(game_map: &GameMap, coord: HexCoord, is_boat: bool) -> bool {
    game_map
        .hexes
        .get(&coord)
        .is_some_and(|h| terrain_passable(h.terrain, is_boat))
}

/// Max seconds between two left-clicks on the same hex for them to count as a
/// double-click (select-the-whole-stack, movement phase).
const DOUBLE_CLICK_SECS: f64 = 0.35;

/// Movement points required to enter `coord` for a land unit -- terrain cost
/// from the Terrain Effects Chart (§5.11).  Returns 0 if the hex is off-map or
/// impassable (callers should check passability separately).
fn floor_movement_cost(game_map: &GameMap, coord: HexCoord) -> i16 {
    let Some(tile) = game_map.hexes.get(&coord) else {
        return 0;
    };
    let has_road = coord
        .neighbors()
        .iter()
        .any(|n| game_map.roads.contains(&HexsideRef::new(coord, *n)));
    omdurman_rules::terrain_chart::movement_cost_with_road(tile.terrain, has_road)
        .map_or(0, |c| c.value() as i16)
}

/// Remaining movement points for a placed unit this turn, from the rules
/// engine: full (night-adjusted) allowance minus what the unit already spent
/// (§5.11/§5.12). Units with no rules identity, or in a session with no game
/// state (editor), are treated as unconstrained (99). This is the budget the
/// picker plots movement against for both single and stack selection.
fn unit_remaining_mp(
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
                let up_left = (g.upstream.value() as i16 - spent).max(0);
                let down_left = (g.downstream.value() as i16 - spent).max(0);
                up_left.max(down_left)
            }
            _ => 99,
        }
    } else {
        99
    }
}

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
    let k = idx as f32 - (n as f32 - 1.0) / 2.0;
    Vec3::new(k * spread, 0.0, k * spread * 0.6)
}

/// Among the units occupying `coord`, pick the one whose rendered position
/// (hex centre + `stack_offset`) is nearest the cursor `hit` point -- so
/// clicking a stacked hex selects the specific counter under the cursor, not
/// always the first in the stack. `center` is the hex's world position;
/// `spread` should match the rendered spread (use the expanded value, since a
/// hex is hovered when clicked).
fn nearest_placed_unit_at<'a>(
    units: &'a Query<(Entity, &PlacedUnit)>,
    coord: HexCoord,
    center: Vec3,
    hit: Vec3,
    spread: f32,
) -> Option<(Entity, &'a PlacedUnit)> {
    let mut stack: Vec<(Entity, &PlacedUnit)> = units
        .iter()
        .filter(|(_, u)| u.coord == coord)
        .collect();
    stack.sort_by_key(|(e, _)| e.to_bits());
    let n = stack.len();
    let mut best: Option<(usize, f32)> = None;
    for (i, _) in stack.iter().enumerate() {
        let off = stack_offset(i, n, spread);
        let d = (hit.x - (center.x + off.x)).powi(2)
            + (hit.z - (center.z + off.z)).powi(2);
        match best {
            Some((_, bd)) if d >= bd => {}
            _ => best = Some((i, d)),
        }
    }
    best.and_then(|(i, _)| stack.get(i).copied())
}

// -- Resources ------------------------------------------------------------------

#[derive(Resource, Clone)]
pub struct UnitPicker {
    pub available: Vec<PickerUnit>,
    pub all: Vec<(SectionName, u32, u32, Handle<Image>, bool)>,
    /// When true (the default), placing a unit automatically selects the next
    /// available unit in the same section so the player can keep clicking to
    /// place multiples without returning to the picker panel.
    pub auto_place_next: bool,
}

impl Default for UnitPicker {
    fn default() -> Self {
        Self {
            available: Vec::new(),
            all: Vec::new(),
            auto_place_next: true,
        }
    }
}

impl UnitPicker {
    pub fn reset_available(&mut self) {
        self.available = self
            .all
            .iter()
            .map(|(sn, col, row, handle, is_boat)| PickerUnit {
                section_name: *sn,
                col: *col,
                row: *row,
                handle: handle.clone(),
                is_boat: *is_boat,
                visible: true,
                egui_texture: None,
                annotations_loaded: false,
            })
            .collect();
    }
}

#[derive(Clone)]
pub struct PickerUnit {
    pub section_name: SectionName,
    pub col: u32,
    pub row: u32,
    pub handle: Handle<Image>,
    pub is_boat: bool,
    pub visible: bool,
    pub egui_texture: Option<egui::TextureHandle>,
    pub annotations_loaded: bool,
}

#[derive(Resource, Default, Clone)]
pub enum PickerState {
    #[default]
    Idle,
    Placing {
        unit_idx: usize,
        preview_hex: Option<HexCoord>,
        preview_valid: bool,
        drag_drop: bool,
    },
    /// A friendly unit has been selected.  Actions:
    /// * Left-click on reachable hex -> extend path annotation
    /// * Confirm (Enter / button) -> commit full path
    /// * Right-click -> deselect
    Selected {
        source: Entity,
        start_coord: HexCoord,
        remaining_mp: i16,
        /// Once a leg has entered an enemy ZOC (§5.43), the unit must stop and
        /// may not extend the path further this turn. The player can still
        /// commit the path built so far.
        forced_stop: bool,
    },
    /// A double-click in the movement phase selected *every* unit in a hex as
    /// one group. The whole group follows a single plotted path; units with
    /// less remaining movement drop off (stop) as soon as the next leg would
    /// exceed their budget, and on commit each unit is moved along the longest
    /// prefix of the path it can afford (§stack-move). Once the move is
    /// committed, the units are independent again.
    SelectedStack(StackSelection),
}

/// The group of units selected by a movement-phase double-click on their hex.
///
/// `sources` and the two movement vectors are parallel and kept in stable
/// (entity-id) order. While a leg is being plotted, only units whose remaining
/// movement covers the leg's cost are charged; a unit that can't afford the
/// next leg keeps its remaining budget (it has "dropped" and will stop at the
/// last affordable hex, recomputed from its budget at commit).
#[derive(Clone, PartialEq)]
pub struct StackSelection {
    /// Every selected unit, sharing one hex.
    pub sources: Vec<Entity>,
    /// The group's planned position: the start hex, advancing with each
    /// plotted leg.
    pub start_coord: HexCoord,
    /// Per-unit remaining movement this turn, parallel to `sources`.
    pub remaining_mp: Vec<i16>,
    /// Each unit's remaining movement when the stack was selected -- needed to
    /// refund a popped leg exactly on undo.
    pub initial_mp: Vec<i16>,
    /// Sticky once any plotted leg enters an enemy ZOC (§5.43): the group may
    /// not extend the path further this turn.
    pub forced_stop: bool,
}

/// Owned snapshot of the picker state driving a click or hotkey. Copied out
/// of `PickerState` before the match so the arms can re-borrow it mutably
/// (to hand `&mut` back into a [`SelectedClick`] / [`SelectedStackClick`]
/// or to build a replacement state) without aliasing the match scrutinee.
enum ActiveSelection {
    Idle,
    Placing { unit_idx: usize, drag_drop: bool },
    Single {
        source: Entity,
        start_coord: HexCoord,
        remaining_mp: i16,
        forced_stop: bool,
    },
    Stack(StackSelection),
}

impl ActiveSelection {
    fn snapshot(state: &PickerState) -> ActiveSelection {
        match state {
            PickerState::Idle => ActiveSelection::Idle,
            PickerState::Placing { unit_idx, drag_drop, .. } => ActiveSelection::Placing {
                unit_idx: *unit_idx,
                drag_drop: *drag_drop,
            },
            PickerState::Selected {
                source,
                start_coord,
                remaining_mp,
                forced_stop,
            } => ActiveSelection::Single {
                source: *source,
                start_coord: *start_coord,
                remaining_mp: *remaining_mp,
                forced_stop: *forced_stop,
            },
            PickerState::SelectedStack(sel) => ActiveSelection::Stack(sel.clone()),
        }
    }
}

/// Accumulated multi-leg movement path while a unit is selected.
///
/// Each leg is a `(from, to)` pair; the first leg's `from` is the unit's
/// original position.  Legs are accumulated locally but *not* committed
/// until the player confirms.  On confirm the full path is sent as a
/// single [`GameEvent::MoveUnit`] and the path is cleared.
///
/// The turn-path shadow (translucent mesh) is rendered from this resource
/// while it is populated, and persists after confirmation via
/// [`UnitPaths`].
#[derive(Resource, Default, Clone)]
pub struct MovementPath {
    pub legs: Vec<(HexCoord, HexCoord)>,
    pub cost_so_far: i16,
}

impl MovementPath {
    /// The final hex of the path (the unit's planned destination), or
    /// `None` if no legs have been added yet.
    pub fn current_end(&self) -> Option<HexCoord> {
        self.legs.last().map(|(_, to)| *to)
    }

    /// Clear the path (called on confirm, deselect, or new selection).
    pub fn reset(&mut self) {
        self.legs.clear();
        self.cost_so_far = 0;
    }
}

// -- Components -----------------------------------------------------------------

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

/// Marker present on the currently-selected unit entity. Allows ECS queries
/// like `Query<&PlacedUnit, With<Selected>>` without touching `PickerState`.
#[derive(Component)]
pub struct Selected;

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

/// Marker for one arrow segment of a unit's movement path. The whole arrow set
/// is rebuilt (see [`movement_path_arrows`]) whenever the paths or the hovered
/// hex change, and each segment is spawned with the dim or bright material
/// already chosen for its path's hover state -- so no per-segment path data
/// needs to live on the entity.
#[derive(Component)]
pub(crate) struct MovementPathArrow;

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

/// Bucket a sprite-sheet cell `(filename, col, row)` into its section.
///
/// Matches the *complete* cell name (`Hadendowa_Forts_0_0`) rather than a
/// prefix: a prefix match would swallow the `Hadendowa_Forts` block into
/// `Hadendowa` (both names start with `Hadendowa_`), silently dropping the
/// fort counters from the picker and with them the auto-setup North Fort
/// placement (§9.344).
fn bucket_section(order: &[SectionName], filename: &str, col: u32, row: u32) -> Option<SectionName> {
    order
        .iter()
        .find(|s| format!("{}_{}_{}", s, col, row) == filename)
        .copied()
}

pub fn spawn_picker_assets(mut picker: ResMut<UnitPicker>, asset_server: Res<AssetServer>) {
    let order = section_order();

    let mut section_sprites: Vec<Vec<PickerUnit>> = order.iter().map(|_| Vec::new()).collect();

    for &(filename, col, row) in generated::SPRITE_PATHS {
        if let Some(section_name) = bucket_section(order, filename, col, row) {
            let idx = order.iter().position(|s| *s == section_name).unwrap();
            let path = format!("sprites/{}.webp", filename);
            let handle = asset_server.load(&path);
            section_sprites[idx].push(PickerUnit {
                section_name,
                col,
                row,
                handle,
                is_boat: false,
                visible: true,
                egui_texture: None,
                annotations_loaded: false,
            });
        }
    }

    for sprites in section_sprites {
        for sprite in sprites {
            picker.all.push((
                sprite.section_name,
                sprite.col,
                sprite.row,
                sprite.handle.clone(),
                sprite.is_boat,
            ));
            picker.available.push(sprite);
        }
    }
}

// -- Left sidebar: list available units -----------------------------------------

fn load_egui_texture(
    ctx: &egui::Context,
    image: &Image,
    label: &str,
) -> Option<egui::TextureHandle> {
    let w = image.width() as usize;
    let h = image.height() as usize;
    if w == 0 || h == 0 {
        return None;
    }
    let data = image.data.as_ref()?;
    if data.len() < w * h * 4 {
        return None;
    }
    let pixels: Vec<egui::Color32> = data
        .chunks(4)
        .take(w * h)
        .map(|c| egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
        .collect();
    if pixels.len() != w * h {
        return None;
    }
    let color_image = egui::ColorImage {
        size: [w, h],
        pixels,
        source_size: egui::vec2(w as f32, h as f32),
    };
    Some(ctx.load_texture(label, color_image, egui::TextureOptions::LINEAR))
}

/// Render the visible picker units belonging to `faction`, grouped by section
/// with a label per section and a wrapped grid of sprite cells. Records a click
/// or drag-start into `clicked_idx` / `drag_idx` (an index into
/// `picker.available`). Shared by both faction categories in the picker.
///
/// When `annotations` is `Some`, each sprite cell gains a hover tooltip
/// showing the counter's resolved profile -- identity (e.g. "1B 1st Btn"),
/// fire/melee/movement factors, weapon class, and the rulebook paragraph for
/// its section. The tooltip is informational; clicking still picks the unit.
/// Bundle of `&UnitPicker` + `&PickerState` so [`render_faction_units`] stays
/// under clippy's argument limit. Plain struct (the consumer is not a system).
struct PickerRead<'a> {
    picker: &'a UnitPicker,
    state: &'a PickerState,
}

/// Bundle of the `clicked_idx` + `drag_idx` out-parameters so
/// [`render_faction_units`] stays under clippy's argument limit.
struct DragState<'a> {
    clicked_idx: &'a mut Option<usize>,
    drag_idx: &'a mut Option<usize>,
}

/// Bundle of the optional annotations + the rulebook reference so
/// [`render_faction_units`] stays under clippy's argument limit.
struct UnitAnnotations<'a> {
    annotations: Option<&'a SpriteAnnotationsResource>,
    rulebook: &'a crate::rulebook::Rulebook,
}

/// Bundle of the image assets + sprite annotations + rulebook reference
/// consumed by [`unit_picker_ui`], so the system stays under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PickerAssetCtx<'w> {
    pub images: Res<'w, Assets<Image>>,
    pub annotations: Option<Res<'w, SpriteAnnotationsResource>>,
    pub rulebook: Res<'w, crate::rulebook::Rulebook>,
}

/// Bundle of the picker + picker-state + game-map resources consumed by
/// [`placement_preview_mesh`], so the system stays under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct PickerPlacementState<'w> {
    pub picker: Res<'w, UnitPicker>,
    pub state: ResMut<'w, PickerState>,
    pub game_map: Res<'w, GameMap>,
}

/// Bundle of the window + camera queries used by [`placement_preview_mesh`] and
/// other picker mesh systems, so their signatures stay under Bevy's
/// system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct WindowCameraQuery<'w, 's> {
    pub windows: Query<'w, 's, &'static Window>,
    pub cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<RtsCamera>>,
}

/// Bundle of `&mut PickerState` + `&mut Commands` for [`handle_idle_click`].
/// Plain struct (the consumer is not a system).
struct IdleSelectionCtx<'a, 'b, 'c> {
    state: &'a mut PickerState,
    commands: &'a mut Commands<'b, 'c>,
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

fn render_faction_units(
    ui: &mut egui::Ui,
    picker: PickerRead,
    faction: omdurman_types::Player,
    cell_size: f32,
    sprite_size: f32,
    drag: DragState,
    ctx: UnitAnnotations,
) {
    let PickerRead { picker, state } = picker;
    let DragState {
        clicked_idx,
        drag_idx,
    } = drag;
    let UnitAnnotations {
        annotations: _annotations,
        rulebook,
    } = ctx;
    let mut current_section = None::<SectionName>;
    for idx in 0..picker.available.len() {
        if !picker.available[idx].visible {
            continue;
        }
        let section_name = picker.available[idx].section_name;
        if omdurman_rules::unit_profiles::section_owner(section_name) != Some(faction) {
            continue;
        }
        if Some(section_name) != current_section {
            current_section = Some(section_name);
            // Count how many counters in this section remain unplaced, so the
            // player can track deployment progress per block (e.g. "32×
            // Mulazmin") rather than only watching the tray empty.
            let remaining = picker
                .available
                .iter()
                .filter(|u| u.visible && u.section_name == section_name)
                .count();
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(section_name.display_name())
                        .size(13.0)
                        .color(egui::Color32::from_gray(180)),
                );
                ui.label(
                    egui::RichText::new(format!("({remaining})"))
                        .size(11.0)
                        .color(egui::Color32::from_gray(120)),
                );
            });
            ui.add_space(2.0);

            ui.horizontal_wrapped(|ui| {
                for j in idx..picker.available.len() {
                    if Some(picker.available[j].section_name) != current_section {
                        break;
                    }
                    if !picker.available[j].visible {
                        continue;
                    }
                    let is_selected =
                        matches!(&*state, PickerState::Placing { unit_idx, .. } if *unit_idx == j);
                    let unit = &picker.available[j];

                    let (rect, response) = ui.allocate_exact_size(
                        egui::Vec2::new(cell_size, cell_size),
                        egui::Sense::click_and_drag(),
                    );

                    let bg = if is_selected {
                        egui::Color32::from_rgb(120, 80, 30)
                    } else if response.hovered() {
                        egui::Color32::from_rgb(80, 65, 45)
                    } else {
                        egui::Color32::from_gray(35)
                    };
                    let painter = ui.painter();
                    painter.rect_filled(rect, 3.0, bg);

                    if let Some(tex_id) = unit.egui_texture.as_ref().map(|t| t.id()) {
                        let img_rect = egui::Rect::from_center_size(
                            rect.center(),
                            egui::Vec2::new(sprite_size, sprite_size),
                        );
                        painter.image(
                            tex_id,
                            img_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    } else {
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}x{}", unit.col, unit.row),
                            egui::FontId::proportional(10.0),
                            egui::Color32::from_gray(120),
                        );
                    }

                    if response.clicked() {
                        *clicked_idx = Some(j);
                    }
                    if response.drag_started() {
                        *drag_idx = Some(j);
                    }

                    // Hover tooltip: the counter's resolved profile, sourced
                    // from the compiled annotations data + the rules engine's
                    // section classifier. Plain text (egui tooltips are
                    // non-interactive by default), with the rulebook citation
                    // rendered as a titled reference via `Rulebook::title_of`
                    // -- the player sees "§2.32 Anglo-Egyptian weapon types"
                    // rather than a bare section number.
                    let unit_id = unit_id_for_section_pos(
                        unit.section_name,
                        unit.col as u8,
                        unit.row as u8,
                    );
                    let profile = unit_id
                        .and_then(omdurman_rules::unit_profiles::profile_for_unit);
                    response.on_hover_ui(|ui| {
                        draw_picker_tooltip(
                            ui,
                            unit.section_name,
                            unit.col,
                            unit.row,
                            unit_id,
                            profile.as_ref(),
                            rulebook,
                        );
                    });
                }
            });
        }
    }
}

/// Render the hover tooltip for one picker sprite. Plain text + a titled §
/// reference; not interactive (egui's `on_hover_ui` tooltip closes on cursor
/// exit, so deep-links would be fiddly -- the rulebook tab is one click away
/// via the chart sheet for players who want to read more).
fn draw_picker_tooltip(
    ui: &mut egui::Ui,
    section_name: SectionName,
    col: u32,
    row: u32,
    unit_id: Option<UnitId>,
    profile: Option<&omdurman_rules::UnitProfile>,
    rulebook: &crate::rulebook::Rulebook,
) {
    ui.set_max_width(240.0);
    // Identity header.
    let identity_str = if let Some(p) = profile {
        p.identity.short_label()
    } else {
        format!("{} ({}x{})", section_name.display_name(), col, row)
    };
    ui.label(
        egui::RichText::new(identity_str)
            .color(egui::Color32::from_rgb(0x1A, 0x16, 0x10))
            .strong()
            .size(13.0),
    );

    // Factors + weapon + movement.
    if let Some(p) = profile {
        let factors = format!(
            "fire {}  ·  melee {}  ·  {}",
            p.fire.map(|f| f.value().to_string()).unwrap_or("—".into()),
            p.melee.map(|m| m.value().to_string()).unwrap_or("—".into()),
            movement_short(&p.movement),
        );
        ui.colored_label(egui::Color32::from_rgb(0x6B, 0x62, 0x50), factors);
        ui.colored_label(egui::Color32::from_rgb(0x6B, 0x62, 0x50), format!("weapon: {}", p.weapon));
        ui.colored_label(egui::Color32::from_rgb(0x6B, 0x62, 0x50), format!("kind: {:?}", p.kind));
        // Printed counter text (e.g. "1B", "Khalifa") and the second-fire
        // flag -- facts the rules profile doesn't carry but the player can
        // see on the counter itself.
        let text = unit_id.map(UnitId::text).unwrap_or("");
        if !text.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(0x6B, 0x62, 0x50), format!("“{text}”"));
        }
        if unit_id.is_some_and(|id| id.kind().is_some_and(|k| k.fires_twice())) {
            ui.colored_label(egui::Color32::from_rgb(0x6B, 0x62, 0x50), "fires twice per phase (§6.42)");
        }
    } else {
        ui.colored_label(
            egui::Color32::from_rgb(0x6B, 0x62, 0x50),
            "no profile resolved for this counter",
        );
    }

    // Rulebook citation for the section, annotated with its title.
    let paragraph = section_paragraph(section_name);
    let title = rulebook.title_of(paragraph);
    let citation = if let Some(t) = title {
        format!("§{paragraph} {t}")
    } else {
        format!("§{paragraph}")
    };
    ui.add_space(2.0);
    ui.colored_label(egui::Color32::from_rgb(0x6B, 0x62, 0x50), citation);
}

fn movement_short(m: &omdurman_rules::UnitMovement) -> String {
    match m {
        omdurman_rules::UnitMovement::Land(a) => format!("move {}", a.value()),
        omdurman_rules::UnitMovement::Gunboat(g) => {
            format!("gunboat {}↑/{}↓", g.upstream.value(), g.downstream.value())
        }
        omdurman_rules::UnitMovement::Immobile => "immobile".into(),
    }
}

/// The rulebook section that documents a sprite-sheet section. Used by the
/// picker tooltip to deep-link the player to the right paragraph for the
/// counter they're hovering.
fn section_paragraph(section_name: SectionName) -> &'static str {
    use SectionName::*;
    match section_name {
        // Dervish leaders and tribes (§2.31).
        KhalifaAbdullah | Sherif | AliWadHelu | SheikElDin | Yakub | OsmanDigna => "2.31",
        Taiasha | Hadendowa | Baggara | Jehadia | Mulazmin | Kehena | Degheim | Danagla
        | UpperJaalin | LowerJaalin => "2.31",
        HadendowaForts => "2.31",
        // Anglo-Egyptian units (§2.32).
        BritishArmy | EgyptianArmy | Kitchener | BritishBoats => "2.32",
        UpperGreen | LowerGreen => "2.32",
    }
}

pub fn unit_picker_ui(
    mut contexts: EguiContexts,
    mode: Res<State<crate::AppMode>>,
    mut picker_ctx: PickerContext,
    peers: crate::peers::Peers,
    assets: PickerAssetCtx,
    game_state: Option<Res<crate::GameStateResource>>,
    mut was_game_started: Local<bool>,
) {
    let PickerAssetCtx {
        images,
        annotations,
        rulebook,
    } = assets;
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_play() {
        return;
    }
    // Spectators have no units to place -- hide the picker entirely so they
    // can't enter a placement (the click handler also rejects it defensively).
    if peers.is_spectator() {
        return;
    }

    // -- cache egui textures & look up is_boat from annotations --
    for unit in &mut picker_ctx.picker.available {
        if unit.egui_texture.is_none()
            && let Some(image) = images.get(&unit.handle)
        {
            let label = format!("picker_{}_{}_{}", unit.section_name, unit.col, unit.row);
            unit.egui_texture = load_egui_texture(ctx, image, &label);
        }
        if !unit.annotations_loaded {
            if (!unit.is_boat || unit.visible)
                && let Some(ref ann) = annotations
            {
                let entry = ann
                    .0
                    .get(&unit.section_name)
                    .and_then(|m| m.get(&(unit.col, unit.row)));
                if let Some(a) = entry {
                    if a.is_boat() {
                        unit.is_boat = true;
                    }
                    if !a.is_unit() {
                        unit.visible = false;
                    }
                }
            }
            // Fallback to compiled sprite data when no annotation entry exists
            // for this position. Hide non-placeable cells -- turn counters,
            // section labels, §6.63 wall-breach markers, bare colour counters
            // -- so they never appear in the picker (and especially not during
            // setup). A cell is placeable iff it resolves to a unit profile;
            // Marker / Breech / BareCounter cells all resolve to `None`.
            if unit.visible {
                let placeable = unit_id_for_section_pos(
                    unit.section_name,
                    unit.col as u8,
                    unit.row as u8,
                )
                .and_then(omdurman_rules::unit_profiles::profile_for_unit)
                .is_some();
                if !placeable {
                    unit.visible = false;
                }
            }
            unit.annotations_loaded = true;
        }
    }

    // -- scenario-based visibility filter --
    // Hide units whose section is not part of the active scenario's order of
    // battle, and hide named gunboats in FoK (§9.321 — only old gunboats).
    if let Some(state) = game_state.as_deref() {
        if let Some(allowed) = state.0.scenario.sections_for_picker() {
            for unit in &mut picker_ctx.picker.available {
                if !allowed.contains(&unit.section_name) {
                    unit.visible = false;
                }
            }
        }
        if matches!(state.0.scenario, Scenario::FallOfKhartoum) {
            for unit in &mut picker_ctx.picker.available {
                if !unit.visible {
                    continue;
                }
                // §9.321: only the two old (unnamed) gunboats are in play in
                // FoK. The named-vs-old distinction lives on the unit's
                // *identity* (`GunboatId::Named` vs `GunboatId::Old`), not its
                // `kind` -- the `british_boats` resolver tags both as
                // `UnitKind::Gunboat`. Resolve via `profile_for_unit` (compiled
                // sprite data), not annotations, which are unreliable here.
                let is_named = unit_id_for_section_pos(
                    unit.section_name,
                    unit.col as u8,
                    unit.row as u8,
                )
                .and_then(omdurman_rules::unit_profiles::profile_for_unit)
                .is_some_and(|p| {
                    matches!(
                        p.identity,
                        omdurman_rules::UnitIdentity::AngloEgyptianGunboat(
                            omdurman_rules::GunboatId::Named(_)
                        )
                    )
                });
                if is_named {
                    unit.visible = false;
                }
            }
        }
    }

    // -- faction filter (bound multiplayer, setup only) --
    // In a bound game each side deploys only its own counters: hide units whose
    // owner isn't the local player during Phase::Setup (§9.2/§9.3). This keeps
    // the wrong side's counters out of sight; the engine's DeployUnit/
    // RemoveDeployedUnit checks backstop it. Unbound sessions (no faction
    // binding, `local` is `None`) and non-setup phases stay permissive so solo
    // testing can drive both sides.
    if let (Some(local), Some(state)) = (peers.local(), game_state.as_deref()) {
        if matches!(state.0.phase, omdurman_rules::Phase::Setup) {
            for unit in &mut picker_ctx.picker.available {
                let owner_is_local = omdurman_rules::unit_profiles::section_owner(unit.section_name)
                    .is_some_and(|owner| owner == local);
                if !owner_is_local {
                    unit.visible = false;
                }
            }
        }
    }

    let mut __ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("picker_panel"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    // -- sidebar --
    egui::Panel::left("unit_picker_panel")
        .resizable(true)
        .default_size(200.0)
        .size_range(140.0..=320.0)
        .frame(
            egui::Frame::default()
                .fill(crate::ui::panel_bg())
                .inner_margin(egui::Margin::symmetric(8, 8)),
        )
        .show(&mut __ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));
            ui.label(
                egui::RichText::new("Unit Picker")
                    .size(16.0)
                    .color(egui::Color32::from_gray(220)),
            );
            ui.separator();
            ui.add_space(4.0);

            // Auto-place-next toggle: when enabled, placing a unit
            // automatically selects the next one in the same section.
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Auto next")
                        .size(12.0)
                        .color(egui::Color32::from_gray(160)),
                );
                ui.checkbox(&mut picker_ctx.picker.auto_place_next, "");
            });
            ui.add_space(2.0);

            if picker_ctx.picker.available.is_empty() {
                ui.colored_label(egui::Color32::from_gray(140), "all units placed");
            }

            let mut clicked_idx: Option<usize> = None;
            let mut drag_idx: Option<usize> = None;
            let sprite_size = 44.0;
            let margin = 2.0;
            let cell_size = sprite_size + margin * 2.0;

            // clear selection if the picked unit is now invisible
            if let PickerState::Placing { unit_idx, .. } = &*picker_ctx.state
                && picker_ctx.picker.available.get(*unit_idx).is_some_and(|u| !u.visible)
            {
                *picker_ctx.state = PickerState::Idle;
            }

            // Once a game starts, default-open the local player's faction and
            // collapse the other. This is a local view choice -- afterwards the
            // user may fold/unfold either heading freely, and nothing is sent
            // over the network.
            let local_faction = peers.local();
            let game_started = peers.any_assigned();

            ui.style_mut().spacing.scroll.floating = false;
            egui::ScrollArea::vertical()
                .id_salt("unit_picker_scroll")
                .show(ui, |ui| {
                    use omdurman_types::Player;
                    // On the transition into a started game, force each category
                    // open/closed once: the local faction open, the foreign one
                    // collapsed. `default_open` alone wouldn't do this, because
                    // egui persists the header's open state from before the game
                    // (when both were open), so we set it explicitly on the edge.
                    let just_started = game_started && !*was_game_started;
                    *was_game_started = game_started;

                    for (faction, heading) in [
                        (Player::Dervish, "Dervish"),
                        (Player::AngloEgyptian, "Anglo-Egyptian"),
                    ] {
                        // Skip a category with no visible units.
                        let any_visible = picker_ctx.picker.available.iter().any(|u| {
                            u.visible
                                && omdurman_rules::unit_profiles::section_owner(u.section_name)
                                    == Some(faction)
                        });
                        if !any_visible {
                            continue;
                        }

                        let header_id = ui.make_persistent_id(("picker_faction", heading));
                        let mut header =
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                header_id,
                                true,
                            );
                        // Force open/closed at the game-start edge.
                        if just_started {
                            header.set_open(local_faction == Some(faction));
                        }
                        header
                            .show_header(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(heading)
                                        .size(14.0)
                                        .color(egui::Color32::from_gray(210)),
                                );
                            })
                            .body(|ui| {
                                render_faction_units(
                                    ui,
                                    PickerRead {
                                        picker: &picker_ctx.picker,
                                        state: &picker_ctx.state,
                                    },
                                    faction,
                                    cell_size,
                                    sprite_size,
                                    DragState {
                                        clicked_idx: &mut clicked_idx,
                                        drag_idx: &mut drag_idx,
                                    },
                                    UnitAnnotations {
                                        annotations: annotations.as_deref(),
                                        rulebook: &rulebook,
                                    },
                                );
                            });
                    }
                });

            if let Some(idx) = clicked_idx {
                match &*picker_ctx.state {
                    PickerState::Placing { unit_idx, .. } if *unit_idx == idx => {
                        *picker_ctx.state = PickerState::Idle;
                    }
                    _ => {
                        *picker_ctx.state = PickerState::Placing {
                            unit_idx: idx,
                            preview_hex: None,
                            preview_valid: false,
                            drag_drop: false,
                        };
                    }
                }
            }
            if let Some(idx) = drag_idx {
                *picker_ctx.state = PickerState::Placing {
                    unit_idx: idx,
                    preview_hex: None,
                    preview_valid: false,
                    drag_drop: true,
                };
            }
        });

    // -- ghost sprite at cursor when placing --
    if let PickerState::Placing { unit_idx, .. } = &*picker_ctx.state
        && let Some(unit) = picker_ctx.picker.available.get(*unit_idx)
        && let Some(tex_id) = unit.egui_texture.as_ref().map(|t| t.id())
        && let Some(pos) = ctx.pointer_latest_pos()
    {
        let ghost_size = 48.0;
        let ghost_rect = egui::Rect::from_center_size(pos, egui::Vec2::new(ghost_size, ghost_size));
        ctx.debug_painter().image(
            tex_id,
            ghost_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::from_white_alpha(180),
        );
    }
}

// -- Placement preview: green/red hex highlight ---------------------------------

#[derive(Component)]
pub(crate) struct PreviewHexRing;

pub fn placement_preview_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    picker_state: PickerPlacementState,
    win_cam: WindowCameraQuery,
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
    let WindowCameraQuery { windows, cameras } = win_cam;
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

    let Some(hit) = raycast_ground(&windows, &cameras) else {
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
    pub windows: Query<'w, 's, &'static Window>,
    pub cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<RtsCamera>>,
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub action_writer: MessageWriter<'w, events::LocalAction>,
}

pub fn handle_picker_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
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
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.egui_wants_pointer_input() {
        return;
    }

    // §turn-order: a unit may only be moved on its owner's turn. When a game is
    // live, gate interactive movement on the local player being the rules
    // engine's active player (`handle_idle_click`/move path below). Placement
    // during set-up is not gated. With no game state (editor) there is no gate.
    let may_move = game_state.is_none_or(|gs| peers.may_act(gs.0.active_player));

    // In bound multiplayer a player may only pick up their own faction's units;
    // an unbound session / single-seat (no faction bindings) may move
    // either side. `may_move` already gates *that it's the right turn*.
    let restrict_to = if !peers.any_assigned() {
        None
    } else {
        peers.local()
    };

    let Some(hit) = raycast_ground(&picker_ctx.windows, &picker_ctx.cameras) else {
        return;
    };
    let origin = picker_ctx.layout.adjusted_origin(&picker_ctx.overlay.params);
    let coord = hit_to_hex(hit, origin, &picker_ctx.overlay.params);
    // World-space centre of the clicked hex -- used to resolve which counter in
    // a stack is under the cursor (stacks fan out around the centre).
    let center = hex_world_pos(coord, origin, &picker_ctx.overlay.params);
    let stack_spread = 0.34 * picker_ctx.overlay.params.hex_size;

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
        ) {
            let owner = omdurman_rules::unit_profiles::section_owner(placed.section_name);
            if restrict_to.is_some_and(|f| owner != Some(f)) {
                return; // not your unit
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
    // stack for a group move -- movement phase only. The first click of the
    // pair has already run `handle_idle_click` (or the Setup-focus path above),
    // so on the second press we must *override* whatever single-unit selection
    // the first click left behind. Track the last click per-hex; `may_move`
    // gates on the active player, and placement stays untouched.
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
        handle_stack_double_click(
            &mut picker_ctx.state,
            &mut picker_ctx.commands,
            &picker_ctx.placed_units,
            coord,
            game_state,
            restrict_to,
        );
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
                hit,
                center,
                stack_spread,
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
        ActiveSelection::Placing { unit_idx, drag_drop } => {
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
                picker_ctx.action_writer.write(events::LocalAction { event });
            }
        }
        ActiveSelection::Single {
            source,
            start_coord,
            remaining_mp,
            forced_stop,
        } => {
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
                picker_ctx.action_writer.write(events::LocalAction { event });
            }
            if matches!(&*picker_ctx.state, PickerState::Idle) {
                picker_ctx
                    .commands
                    .entity(source)
                    .remove::<Selected>();
            }
        }
        ActiveSelection::Stack(sel) => {
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
                picker_ctx.action_writer.write(events::LocalAction { event });
            }
        }
    }
}

/// Idle: a left-press on a placed unit selects it.  During setup, it removes
/// the unit from the board (re-pickup for re-placement).
///
/// `restrict_to`, when `Some`, is the only faction whose units may be picked
/// up -- set in bound multiplayer so a player can't grab an enemy counter on
/// their own turn.  `None` (unbound session / single-seat) allows selecting
/// either side, so solo play/testing can drive both factions.
fn handle_idle_click(
    pressed: bool,
    coord: HexCoord,
    placed_units: &Query<(Entity, &PlacedUnit)>,
    selection: IdleSelectionCtx,
    game_state: Option<&crate::GameStateResource>,
    restrict_to: Option<omdurman_types::Player>,
    hit: Vec3,
    center: Vec3,
    stack_spread: f32,
) {
    let IdleSelectionCtx {
        state,
        commands,
    } = selection;
    if !pressed {
        return;
    }
    let Some((entity, placed)) =
        nearest_placed_unit_at(placed_units, coord, center, hit, stack_spread)
    else {
        return;
    };

    if let Some(faction) = restrict_to
        && omdurman_rules::unit_profiles::section_owner(placed.section_name) != Some(faction)
    {
        return; // not your unit -- ignore the click
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
) {
    let mut sources: Vec<Entity> = placed_units
        .iter()
        .filter(|(_, u)| u.coord == coord)
        .filter(|(_, u)| match restrict_to {
            Some(faction) => {
                omdurman_rules::unit_profiles::section_owner(u.section_name) == Some(faction)
            }
            None => true,
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
fn deploy_candidate(
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
        let can_place = if game_state
            .is_some_and(|gs| matches!(gs.0.phase, omdurman_rules::Phase::Setup))
        {
            game_state
                .zip(deploy_candidate(self.picker, unit_idx, coord))
                .is_some_and(|(gs, candidate)| gs.0.can_deploy_unit(&candidate).is_ok())
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
            let is_boat = unit_id_for_section_pos(unit.section_name, unit.col as u8, unit.row as u8)
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

/// Borrowed context for resolving a click while a unit is selected.
///
/// As with [`PlacingClick`], the map query is passed to
/// [`handle`](Self::handle) rather than stored, so the invariant `Query`
/// lifetimes never couple to the struct's `Commands` borrow.
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
        let mover_owner = omdurman_rules::unit_profiles::section_owner(placed.section_name);

        // §9.346: a Dervish unit may move *onto* the Palace even though GORDON
        // occupies it -- "passing through or occupying the palace hex" is how he
        // is eliminated. So the normal "can't enter an enemy-occupied hex" gate
        // is waived for the Palace (the faction gate upstream already ensures
        // only the side whose turn it is can reach here; the engine resolves
        // GORDON's death for a Dervish occupant).
        let dest_is_palace = self.game_map.hexes.get(&coord).is_some_and(|h| {
            h.name
                .as_deref()
                .and_then(omdurman_types::Location::from_tile_name)
                == Some(omdurman_types::Location::Palace)
        });
        // A unit may not move into an enemy-occupied hex (that's melee / advance
        // after combat, not movement) -- except the §9.346 palace waiver.
        // Friendly-occupied hexes are allowed: §5.51 lets up to four units (plus
        // free-stacking leaders) share a hex, which `stacking_ok` validates.
        let enemy_occupied = !dest_is_palace
            && mover_owner.is_some()
            && placed_units.iter().any(|(_, u)| {
                u.coord == coord
                    && omdurman_rules::unit_profiles::section_owner(u.section_name) != mover_owner
            });
        // Stacking is enforced by the engine; consult `check_stacking` so the UI
        // pre-rejects a leg that would exceed the 4-unit cap (or break the
        // gunboat / tribe-mix / leader-command rules) instead of letting the
        // player build a path the commit would reject. With no engine state,
        // defer to commit-time validation.
        let stacking_ok = match (placed.unit_id, game_state) {
            (Some(uid), Some(gs)) => gs
                .0
                .find_unit(uid)
                .is_some_and(|mover| gs.0.check_stacking(mover, coord).is_ok()),
            _ => true,
        };
        // §5.43: a unit must stop the instant it enters an enemy ZOC. Detect
        // whether this leg's destination is in an enemy ZOC so the builder can
        // force a stop (no further legs this turn). A unit that *began* in a ZOC
        // may still move out (§5.43), so only the destination matters here.
        // With no engine state, defer to commit-time validation.
        let entering_enemy_zoc = match (placed.unit_id, game_state) {
            (Some(uid), Some(gs)) => gs
                .0
                .find_unit(uid)
                .is_some_and(|mover| {
                    gs.0.hex_in_enemy_zoc(
                        coord,
                        mover.profile.identity.owner(),
                        mover.profile.kind,
                    )
                }),
            _ => false,
        };
        // Adjacency is checked against the *planned* current position
        // (start_coord), not the unit's original placed.coord, so
        // multi-leg path building works correctly.
        let adjacent = start_coord.neighbors().contains(&coord);
        let passable = coord_passable(self.game_map, coord, placed.is_boat);
        let cost = if adjacent {
            floor_movement_cost(self.game_map, coord)
        } else {
            0
        };
        let affordable = cost > 0 && self.remaining_mp >= cost;

        if adjacent && !enemy_occupied && passable && affordable && stacking_ok && !self.forced_stop
        {
            let new_remaining = self.remaining_mp - cost;
            info!(
                "path leg accepted: {:?} -> {:?}, cost={}, remaining_mp={}, entering_zoc={}",
                start_coord, coord, cost, new_remaining, entering_enemy_zoc
            );
            // Accumulate the leg in the path resource.
            self.movement_path.legs.push((start_coord, coord));
            self.movement_path.cost_so_far += cost;

            // Stay in Selected so the player can confirm or (if not pinned by
            // ZOC) keep adding legs. §5.43 forces a stop once a leg enters an
            // enemy ZOC; the flag is sticky for the rest of the selection.
            *self.state = PickerState::Selected {
                source,
                start_coord: coord,
                remaining_mp: new_remaining,
                forced_stop: self.forced_stop || entering_enemy_zoc,
            };
            // No GameEvent yet — committed on confirm.
            None
        } else {
            info!(
                source = source.to_bits(),
                adjacent,
                enemy_occupied,
                passable,
                affordable,
                stacking_ok,
                forced_stop = self.forced_stop,
                entering_enemy_zoc,
                cost,
                remaining_mp = self.remaining_mp,
                "path leg rejected",
            );
            *self.state = PickerState::Idle;
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
        let mover_owner = omdurman_rules::unit_profiles::section_owner(placed.section_name);

        // §9.346 Palace waiver: Dervish may occupy GORDON's hex.
        let dest_is_palace = self.game_map.hexes.get(&coord).is_some_and(|h| {
            h.name
                .as_deref()
                .and_then(omdurman_types::Location::from_tile_name)
                == Some(omdurman_types::Location::Palace)
        });
        let enemy_occupied = !dest_is_palace
            && mover_owner.is_some()
            && placed_units.iter().any(|(_, u)| {
                u.coord == coord
                    && omdurman_rules::unit_profiles::section_owner(u.section_name) != mover_owner
            });
        // Stacking pre-check uses the first unit as the group's representative
        // (the engine re-validates each unit's move at commit, so this is a
        // courtesy gate for the common case -- moving a stack into a hex that
        // already pushes past the §5.51 cap).
        let stacking_ok = match (placed.unit_id, game_state) {
            (Some(uid), Some(gs)) => gs
                .0
                .find_unit(uid)
                .is_some_and(|mover| gs.0.check_stacking(mover, coord).is_ok()),
            _ => true,
        };
        // §5.43: stop the group the instant a leg enters an enemy ZOC.
        let entering_enemy_zoc = match (placed.unit_id, game_state) {
            (Some(uid), Some(gs)) => gs
                .0
                .find_unit(uid)
                .is_some_and(|mover| {
                    gs.0.hex_in_enemy_zoc(
                        coord,
                        mover.profile.identity.owner(),
                        mover.profile.kind,
                    )
                }),
            _ => false,
        };
        let adjacent = start_coord.neighbors().contains(&coord);
        let passable = coord_passable(self.game_map, coord, placed.is_boat);
        let cost = if adjacent {
            floor_movement_cost(self.game_map, coord)
        } else {
            0
        };
        // A leg is affordable if at least one unit's budget covers it. Units
        // that can't afford it keep their remaining (they are dropped here).
        let any_affordable = cost > 0 && self.remaining_mp.iter().any(|&mp| mp >= cost);
        let affordable = cost > 0 && any_affordable;

        if adjacent && !enemy_occupied && passable && affordable && stacking_ok && !self.forced_stop
        {
            let new_remaining: Vec<i16> = self
                .remaining_mp
                .iter()
                .map(|&mp| if mp >= cost { mp - cost } else { mp })
                .collect();
            info!(
                "stack path leg accepted: {:?} -> {:?}, cost={}, affordable_units={}, entering_zoc={}",
                start_coord,
                coord,
                cost,
                new_remaining
                    .iter()
                    .zip(&self.remaining_mp)
                    .filter(|(nr, mp)| **nr < **mp)
                    .count(),
                entering_enemy_zoc,
            );
            self.movement_path.legs.push((start_coord, coord));
            self.movement_path.cost_so_far += cost;
            *self.state = PickerState::SelectedStack(StackSelection {
                sources: sources.to_vec(),
                start_coord: coord,
                remaining_mp: new_remaining,
                initial_mp: self.initial_mp.clone(),
                forced_stop: self.forced_stop || entering_enemy_zoc,
            });
            None
        } else {
            info!(
                adjacent,
                enemy_occupied,
                passable,
                affordable,
                stacking_ok,
                forced_stop = self.forced_stop,
                entering_enemy_zoc,
                cost,
                "stack path leg rejected",
            );
            *self.state = PickerState::Idle;
            for &source in sources {
                self.commands.entity(source).remove::<Selected>();
            }
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
            for &(_, to) in &self.movement_path.legs {
                let leg_cost = floor_movement_cost(self.game_map, to);
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
        // independent counter again.
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
        movement_path.reset();
    }
}

/// Confirm a pending movement path when the player presses Enter.
///
/// Reads keyboard input and, if a path is pending, fires the commit.
pub(crate) fn confirm_movement_path(
    keys: Res<ButtonInput<KeyCode>>,
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
        .is_some_and(|gs| !peers.may_act(gs.0.active_player))
    {
        return;
    }
    let origin = picker_ctx.layout.adjusted_origin(&picker_ctx.overlay.params);
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
            if let Some(event) = sel.commit_path(&picker_ctx.placed_units, source) {
                picker_ctx.action_writer.write(events::LocalAction { event });
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
            for event in sel_click.commit_path(&picker_ctx.placed_units, &sel.sources) {
                picker_ctx.action_writer.write(events::LocalAction { event });
            }
        }
        ActiveSelection::Placing { .. } | ActiveSelection::Idle => {}
    }
}

/// Undo the last leg of the pending movement path when the player presses
/// Backspace. Refunds the leg's movement points and steps the planned position
/// back one hex; the path can then be re-extended or committed. Only acts on
/// the owning player's turn, while a unit is selected with a pending path.
pub(crate) fn undo_movement_leg(
    keys: Res<ButtonInput<KeyCode>>,
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
        .is_some_and(|gs| !peers.may_act(gs.0.active_player))
    {
        return;
    }
    // Pop the last leg and refund its cost. `floor_movement_cost` depends only
    // on the destination hex, so recomputing matches what was charged.
    let (from, to) = picker_ctx
        .movement_path
        .legs
        .pop()
        .expect("checked non-empty above");
    let cost = floor_movement_cost(&picker_ctx.game_map, to);
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
    if !peers.may_act(gs.0.active_player) {
        return;
    }
    let Ok((_, placed)) = picker_ctx.placed_units.get(*source) else {
        return;
    };
    let Some(ref mut pending) = pending else {
        return;
    };
    pending.outgoing_broadcast.push(omdurman_net::NetMsg::Game(
        omdurman_net::GameEvent::RemoveUnit {
            sprite: omdurman_types::SpriteRef {
                section_name: placed.section_name,
                col: placed.col,
                row: placed.row,
            },
        },
    ));
    // Deselect; the apply path despawns the entity and returns the counter to
    // the picker.
    picker_ctx.commands.entity(*source).remove::<Selected>();
    *picker_ctx.state = PickerState::Idle;
}

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
    let Ok((camera, camera_transform)) = cameras.single() else { return };
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

        let Ok(screen_pos) = camera
            .world_to_viewport(camera_transform, world_pos_3d)
        else {
            continue;
        };

        let cost_str = game_map
            .hexes
            .get(&to)
            .map(|t| {
                let has_road = from
                    .neighbors()
                    .iter()
                    .any(|n| game_map.roads.contains(&HexsideRef::new(to, *n)));
                let cost = omdurman_rules::terrain_chart::movement_cost_with_road(t.terrain, has_road)
                    .map(|c| c.value())
                    .unwrap_or(0);
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
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180))
                    .corner_radius(3.0)
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .show(ui, |ui| {
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

pub fn movement_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    view: MovementOverlayCtx,
    selection: PickerReadSelection,
    existing: MovementRingQueries,
    peers: crate::peers::Peers,
    mut last_key: Local<Option<MovementOverlayKey>>,
) {
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let MovementOverlayCtx { game_map, game_state } = view;
    let PickerReadSelection {
        state,
        placed_units,
    } = selection;
    let MovementRingQueries {
        existing_green,
        existing_gray,
        existing_zoc,
    } = existing;
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
    let Some((start_coord, budget, is_boat, key)) = (match &*state {
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
    let green: Vec<Entity> = existing_green.iter().collect();
    let gray: Vec<Entity> = existing_gray.iter().collect();
    let zoc_ring: Vec<Entity> = existing_zoc.iter().collect();
    crate::ui::despawn_all(&mut commands, &green);
    crate::ui::despawn_all(&mut commands, &gray);
    crate::ui::despawn_all(&mut commands, &zoc_ring);

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;

    // Compute enemy ZOC hexes for this player.
    let my_player = peers.local().unwrap_or(omdurman_types::Player::AngloEgyptian);
    let enemy = my_player.opponent();
    let enemy_zoc = game_state
        .as_ref()
        .map(|gs| crate::zoc::compute_enemy_zoc(&gs.0, enemy, my_player))
        .unwrap_or_default();

    // BFS from the *planned* current position (start_coord), accumulating
    // terrain costs.  When the path is empty start_coord == placed.coord.
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut green_spawned = 0u32;
    let mut gray_spawned = 0u32;
    let mut zoc_spawned = 0u32;

    queue.push_back((start_coord, 0i16));
    visited.insert(start_coord);

    while let Some((cur, cost_so_far)) = queue.pop_front() {
        for neighbor in cur.neighbors() {
            if visited.contains(&neighbor) {
                continue;
            }
            if placed_units.iter().any(|(_, u)| u.coord == neighbor) {
                continue;
            }
            if !coord_passable(&game_map, neighbor, is_boat) {
                continue;
            }
            // §5.23: wall hexsides block movement (gates/breaches pass).
            if game_map
                .hexside_between(cur, neighbor)
                .is_some_and(|s| s.blocks_movement())
            {
                continue;
            }
            let terrain_cost = floor_movement_cost(&game_map, neighbor);
            if terrain_cost <= 0 {
                continue;
            }
            let new_cost = cost_so_far + terrain_cost;
            if new_cost > budget {
                continue;
            }
            visited.insert(neighbor);

            let is_zoc = enemy_zoc.contains(&neighbor);
            let pos = hex_world_pos(neighbor, origin, &overlay.params);

            if is_zoc {
                // §5.41: ZOC hexes are reachable as path termini but the
                // BFS does not expand from them — show with yellow ring
                // to distinguish from normal reachable hexes.
                commands.spawn((
                    MovementZocRing,
                    Mesh3d(assets.mesh.clone()),
                    MeshMaterial3d(assets.yellow.clone()),
                    Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
                    Visibility::Visible,
                ));
                zoc_spawned += 1;
                // Do NOT enqueue — BFS stops at ZOC boundaries (§5.41).
            } else {
                queue.push_back((neighbor, new_cost));
                let is_adjacent = start_coord.neighbors().contains(&neighbor);
                if is_adjacent {
                    commands.spawn((
                        MovementHexRing,
                        Mesh3d(assets.mesh.clone()),
                        MeshMaterial3d(assets.light_green.clone()),
                        Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
                        Visibility::Visible,
                    ));
                    green_spawned += 1;
                } else {
                    commands.spawn((
                        MovementRangeRing,
                        Mesh3d(assets.mesh.clone()),
                        MeshMaterial3d(assets.gray.clone()),
                        Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
                        Visibility::Visible,
                    ));
                    gray_spawned += 1;
                }
            }
        }
    }

    info!(
        green_spawned,
        gray_spawned,
        zoc_spawned,
        budget,
        "movement_overlay_mesh: done"
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
) {
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
            commands.spawn((
                DeploymentZoneRing,
                Mesh3d(assets.mesh.clone()),
                MeshMaterial3d(assets.light_green.clone()),
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
) {
    let crate::HexRender {
        assets, overlay, ..
    } = hex;
    let sources: Vec<Entity> = match &*state {
        PickerState::Selected { source, .. } => vec![*source],
        PickerState::SelectedStack(sel) => sel.sources.clone(),
        _ => Vec::new(),
    };
    if *last_sources == Some(sources.clone()) {
        return;
    }
    let old: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &old);
    *last_sources = Some(sources.clone());
    let Some(&first) = sources.first() else { return };
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
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    peers: crate::peers::Peers,
    mut hovered: ResMut<crate::HoveredUnit>,
) {
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        hovered.0 = None;
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
    let coord = hit_to_hex(hit, origin, &overlay.params);
    let center = hex_world_pos(coord, origin, &overlay.params);
    let spread = 0.34 * overlay.params.hex_size;
    let local = peers.local();
    let target = nearest_placed_unit_at(&placed_units, coord, center, hit, spread)
        .filter(|(_, p)| match local {
            Some(local) => {
                omdurman_rules::unit_profiles::section_owner(p.section_name) == Some(local)
            }
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
) {
    let crate::HexRender {
        assets, overlay, ..
    } = hex;
    let selected: Vec<Entity> = match &*state {
        PickerState::Selected { source, .. } => vec![*source],
        PickerState::SelectedStack(sel) => sel.sources.clone(),
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
    // so each unit's slot index is deterministic frame to frame.
    let mut by_hex: HashMap<HexCoord, Vec<Entity>> = HashMap::new();
    for (entity, placed, _) in &units {
        by_hex.entry(placed.coord).or_default().push(entity);
    }
    for ents in by_hex.values_mut() {
        ents.sort_by_key(|e| e.to_bits());
    }

    let lerp = (time.delta_secs() * 12.0).min(1.0);

    for (entity, placed, mut transform) in &mut units {
        let stack = &by_hex[&placed.coord];
        let n = stack.len();
        let idx = stack.iter().position(|e| *e == entity).unwrap_or(0);
        let center = hex_world_pos(placed.coord, origin, &overlay.params);

        // Per-index offset. A single unit sits centred; a stack fans along a
        // short diagonal so each counter peeks out from under the one above.
        // Hovering a multi-unit hex doubles the spread for readability.
        let expanded = n > 1 && hovered.0 == Some(placed.coord);
        let spread = if expanded { 0.34 } else { 0.14 } * size;
        let off = stack_offset(idx, n, spread);
        // A tiny per-index height step keeps the quads out of the same plane
        // (no y-fighting); the hovered stack lifts a hair more.
        let y_step = if expanded { 0.12 } else { 0.04 };
        let target = Vec3::new(
            center.x + off.x,
            UNIT_HEIGHT + idx as f32 * y_step,
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

/// Despawn the sprite of any counter the rules engine has eliminated. A placed
/// counter that carries a rules `UnitId` no longer present in `GameState.units`
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

// -- Cancel placement/movement on right-click ----------------------------------

pub fn cancel_placement(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut state: ResMut<PickerState>,
    mut movement_path: ResMut<MovementPath>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.egui_wants_pointer_input()
    {
        return;
    }
    movement_path.reset();
    *state = PickerState::Idle;
}

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

fn clear_gameplay_overlays(
    mut commands: Commands,
    rings: GameplayOverlayEntities<'_, '_>,
    parented: ParentedOverlayEntities<'_, '_>,
) {
    let rings: Vec<Entity> = rings.iter().chain(parented.iter()).collect();
    crate::ui::despawn_all(&mut commands, &rings);
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
    let Some(gs) = game_state else { return };
    let active = gs.0.active_player;
    if *last_active != Some(active) {
        if last_active.is_some() {
            paths.0.clear();
            movement_path.reset();
        }
        *last_active = Some(active);
    }
}

/// Registers all game-domain resources and systems: unit picker, combat
/// (fire/melee/retreat), movement animation, dice rolling, and placement
/// application. Systems are gated via [`crate::GameSet`] (active in map modes).
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // -- Resources ----------------------------------------------
            .insert_resource(UnitPicker::default())
            .insert_resource(PickerState::default())
            .insert_resource(MovementPath::default())
            .insert_resource(UnitPaths::default())
            .insert_resource(crate::zoc::ZocOverlay::default())
            // -- Mode-exit cleanup: leaving a play view (or the game itself)
            //    despawns all gameplay overlay rings, so none linger over the
            //    editor / lobby (the per-frame overlay systems only clean up
            //    while running).
            .add_systems(OnExit(crate::AppMode::Game), clear_gameplay_overlays)
            .add_systems(OnExit(AppState::InGame), clear_gameplay_overlays)
            // -- Startup ------------------------------------------------
            .add_systems(
                Startup,
                (
                    spawn_picker_assets,
                    crate::render::spawn_movement_arrow_assets,
                ),
            )
            // -- Update: gameplay (GameSet) -----------------------------
            .add_systems(
                Update,
                (
                    crate::apply_pending_placement.after(crate::net_socket::handle_socket),
                    (
                        placement_preview_mesh.in_set(crate::GameSet),
                        crate::fire_allocation::handle_fire_allocation_click
                            .in_set(crate::GameSet)
                            .before(handle_picker_clicks),
                        crate::melee::handle_melee_combat
                            .in_set(crate::GameSet)
                            .before(handle_picker_clicks),
                        crate::melee::handle_advance_after_combat
                            .in_set(crate::GameSet)
                            .after(crate::melee::handle_melee_combat)
                            .after(crate::fire_allocation::execute_fire_allocations)
                            .before(handle_picker_clicks),
                        crate::retreat::handle_retreat
                            .in_set(crate::GameSet)
                            .before(handle_picker_clicks),
                        handle_picker_clicks.in_set(crate::GameSet),
                        movement_overlay_mesh.in_set(crate::GameSet),
                        crate::fire::fire_target_overlay_mesh.in_set(crate::GameSet),
                        crate::melee::melee_target_overlay_mesh.in_set(crate::GameSet),
                        crate::retreat::retreat_overlay_mesh.in_set(crate::GameSet),
                        clear_paths_on_turn_change,
                        movement_path_arrows
                            .in_set(crate::GameSet)
                            .after(clear_paths_on_turn_change)
                            .after(crate::apply_pending_placement),
                        deployment_zone_overlay_mesh.in_set(crate::GameSet),
                        crate::fok_entry::fok_entry_overlay_mesh.in_set(crate::GameSet),
                        crate::zoc::zoc_overlay_mesh.in_set(crate::GameSet),
                        animate_unit_movement,
                        layout_stacked_units.after(animate_unit_movement),
                        sync_disrupted_visuals,
                        sync_eliminated_visuals,
                        cancel_placement.in_set(crate::GameSet),
                    ),
                ),
            )
            // -- Selection outline + cursor-hex tint + hover square (separate
            //     block: the big GameSet tuple is at Bevy's schedule-config
            //     arity limit) -----------------------------------------------
            .add_systems(
                Update,
                (
                    selection_outline_mesh.in_set(crate::GameSet),
                    placement_marker_color
                        .in_set(crate::GameSet)
                        .after(placement_preview_mesh),
                    update_hovered_unit
                        .in_set(crate::GameSet)
                        .before(hover_outline_mesh),
                    hover_outline_mesh.in_set(crate::GameSet),
                ),
            )
            // -- Execute fire allocations (separate block to stay under Bevy's
            //     tuple size limit for schedule configs) --------------------
            .add_systems(
                Update,
                crate::fire_allocation::execute_fire_allocations
                    .in_set(crate::GameSet)
                    .after(crate::fire_allocation::handle_fire_allocation_click)
                    .before(handle_picker_clicks),
            )
            // -- Path annotation + fire/melee direction systems ---------------
            .add_systems(
                Update,
                (
                    clear_movement_path_when_idle
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    confirm_movement_path
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    undo_movement_leg
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    delete_selected_unit
                        .in_set(crate::GameSet)
                        .before(handle_picker_clicks),
                    movement_path_shadows
                        .in_set(crate::GameSet)
                        .after(clear_paths_on_turn_change)
                        .after(crate::apply_pending_placement),
                    crate::fire::fire_direction_arrow.in_set(crate::GameSet),
                    crate::melee::melee_direction_arrow.in_set(crate::GameSet),
                    crate::melee::advance_target_overlay_mesh.in_set(crate::GameSet),
                    crate::turn_track_ui::turn_track_gizmos.in_set(crate::GameSet),
                    crate::desertion::detect_desertion_turn.in_set(crate::GameSet),
                    crate::river_placement::handle_optional_rule_click
                        .in_set(crate::GameSet)
                        .after(crate::apply_pending_placement),
                ),
            )
            // -- Egui UI panels -----------------------------------------
            // These in-game side panels run only while actually in a game, so
            // they don't linger over the lobby (the EditorMode can still be a
            // map mode in the lobby, which is an AppState, not a mode).
            .add_systems(
                EguiPrimaryContextPass,
                (
                    unit_picker_ui,
                    crate::fire_allocation::fire_allocation_review_ui,
                    crate::melee::melee_reaction_ui,
                    crate::overview::unit_overview_ui,
                    movement_path_labels.run_if(crate::map_view_active),
                    crate::turn_track_ui::turn_track_labels,
                    crate::desertion::desertion_panel_ui,
                )
                    .run_if(in_state(crate::AppState::InGame)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every sprite file must bucket into exactly one section. The `Hadendowa`
    /// and `Hadendowa_Forts` blocks both start with the same prefix; a naive
    /// `starts_with` match would swallow the fort counters into `Hadendowa`,
    /// which silently drops the auto-setup North Fort placement.
    #[test]
    fn sprite_files_bucket_into_exact_sections() {
        let order = section_order();
        for &(filename, col, row) in generated::SPRITE_PATHS {
            let section = bucket_section(order, filename, col, row);            assert!(
                section.is_some(),
                "sprite {filename} must bucket into exactly one section, got None"
            );
        }
    }

    #[test]
    fn fort_sprites_belong_to_hadendowa_forts() {
        let order = section_order();
        assert_eq!(
            bucket_section(order, "Hadendowa_Forts_0_0", 0, 0),
            Some(SectionName::HadendowaForts)
        );
        assert_eq!(
            bucket_section(order, "Hadendowa_0_0", 0, 0),
            Some(SectionName::Hadendowa)
        );
    }
}

use std::f32::consts::PI;

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout};

use omdurman_hexmap::{hex_ring_mesh, hex_world_pos, hit_to_hex};

// Re-exported for the rest of the app (the definitions live in
// `omdurman-hexmap::plane`).
pub use omdurman_hexmap::{HexOverlay, MapTextureCache};

use omdurman_types::HexCoord;

/// Bevy-side colours of the board overlays (hex rings, cursor markers,
/// arrows, turn-track gizmos, counter tints). The egui chrome palette lives in
/// [`crate::ui::palette`]; these are the 3D-world counterparts, kept in one
/// place so every overlay material pulls its tint from here.
pub mod overlay_palette {
    use bevy::prelude::Color;

    /// Hostile / illegal ring (fire targets, illegal hexes).
    pub const RING_HOSTILE: Color = Color::srgb(1.0, 0.0, 0.0);
    /// Legal / friendly ring.
    pub const RING_LEGAL: Color = Color::srgb(0.0, 1.0, 0.0);
    /// Softer legal ring (secondary highlights).
    pub const RING_LEGAL_SOFT: Color = Color::srgb(0.6, 1.0, 0.6);
    /// Orange ring (Dervish selection, howitzer shell burst).
    pub const RING_ORANGE: Color = Color::srgb(1.0, 0.55, 0.1);
    /// Blue ring (Anglo-Egyptian selection).
    pub const RING_BLUE: Color = Color::srgb(0.25, 0.55, 1.0);
    /// Near-white ring for the unit under the cursor.
    pub const RING_HOVER: Color = Color::srgb(1.0, 0.97, 0.55);
    /// Neutral grey ring.
    pub const RING_NEUTRAL: Color = Color::srgb(0.4, 0.4, 0.4);
    /// Movement range: hexes the selected mover can still reach. Teal, so it
    /// reads against the sepia map and its printed dark hex grid (grey did
    /// not).
    pub const RING_REACH: Color = Color::srgb(0.05, 0.62, 0.72);
    /// Translucent yellow ring / fill.
    pub const RING_YELLOW: Color = Color::srgba(1.0, 0.85, 0.0, 0.4);
    /// Cursor-hex fill when placement is legal.
    pub const MARKER_LEGAL: Color = Color::srgba(0.0, 1.0, 0.0, 0.25);
    /// Cursor-hex fill when placement is illegal.
    pub const MARKER_ILLEGAL: Color = Color::srgba(1.0, 0.0, 0.0, 0.25);
    /// Cursor-hex fill with nothing in hand: a neutral wash (the red one
    /// read as "illegal" over every hex the pointer crossed).
    pub const MARKER_IDLE: Color = Color::srgba(1.0, 0.97, 0.85, 0.2);
    /// Shadow under a planned movement path.
    pub const PATH_SHADOW: Color = Color::srgba(0.45, 0.55, 0.95, 0.18);
    /// Fire-declaration arrow.
    pub const FIRE_ARROW: Color = Color::srgba(0.9, 0.15, 0.1, 0.45);
    /// Idle movement arrow.
    pub const MOVE_ARROW: Color = Color::srgba(0.85, 0.5, 0.2, 0.45);
    /// Hovered movement arrow.
    pub const MOVE_ARROW_HOVER: Color = Color::srgba(0.95, 0.55, 0.2, 0.95);
    /// Combat aim and allocation arrows (fire and melee): the red of the
    /// target rings, so they never read as one more movement trail (both
    /// were orange, and a staged shot looked like a unit's route).
    pub const COMBAT_ARROW: Color = Color::srgba(0.82, 0.1, 0.06, 0.9);
    /// Turn-track grid lines.
    pub const TURN_TRACK_GRID: Color = Color::srgba(0.35, 0.35, 0.35, 0.6);
    /// Turn-track current-turn highlight.
    pub const TURN_TRACK_CURRENT: Color = Color::srgba(1.0, 0.3, 0.2, 0.9);
    /// Tint of a disrupted unit counter.
    pub const COUNTER_DISRUPTED: Color = Color::srgb(0.55, 0.55, 0.55);
}

// -- Render resources -------------------------------------------------------

/// Written every frame by `render::update_selection_marker` with the hex
/// currently under the cursor (or `None` if no valid hex is hovered).
#[derive(Resource, Default, PartialEq, Eq, Debug)]
pub struct HoveredHex(pub Option<HexCoord>);

/// The placed-unit entity currently under the cursor (the specific counter in
/// a stack, resolved by `update_hovered_unit`), or `None`. Drives the bright
/// hover square that previews which unit a click would select.
#[derive(Resource, Default, PartialEq, Eq, Debug)]
pub struct HoveredUnit(pub Option<bevy::prelude::Entity>);

// -- Map plane -----------------------------------------------------------------

// The map plane (spawn + both-texture preload) lives in board-ui; the game
// adds its mesh-picking marker right after spawning (see
// `add_plane_picking`).
pub use omdurman_board_ui::board_store::spawn_map_plane;

/// The board plane participates in mesh picking.
/// Runs at startup, right after [`spawn_map_plane`].
pub fn add_plane_picking(
    mut commands: Commands,
    planes: Query<
        Entity,
        (
            With<omdurman_hexmap::MapPlane>,
            Without<bevy::picking::Pickable>,
        ),
    >,
) {
    for entity in &planes {
        commands
            .entity(entity)
            .try_insert(crate::picking::plane_pickable());
    }
}

// -- Selection marker ----------------------------------------------------------

#[derive(Component)]
pub struct SelectionMarker;

pub fn spawn_selection_marker(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(RegularPolygon::new(1.0, 6)));
    let material = materials.add(StandardMaterial {
        base_color: overlay_palette::MARKER_IDLE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
        Visibility::Hidden,
        SelectionMarker,
    ));
}

/// Moves a translucent hex marker to whichever map hex the cursor is over, and
/// records the hovered hex coordinate in [`HoveredHex`] for the UI. Writes only
/// what changed (`set_if_neq`), so `HoveredHex`, the marker's `Transform` and
/// `Visibility` read as changed only when the hovered hex really moves --
/// systems gated on them (the path arrows) stay idle otherwise.
pub fn update_selection_marker(
    ground: Res<crate::picking::PointerGroundHit>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<SelectionMarker>>,
    mut hovered: ResMut<HoveredHex>,
) {
    let Ok((mut transform, mut visibility)) = marker.single_mut() else {
        hovered.set_if_neq(HoveredHex(None));
        return;
    };
    // `None` while the pointer is over UI, off the board, or unseen by any
    // camera (see `picking::update_pointer_ground_hit`).
    let Some(hit) = **ground else {
        visibility.set_if_neq(Visibility::Hidden);
        hovered.set_if_neq(HoveredHex(None));
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    if game_map.hexes.contains_key(&coord) {
        let pos = hex_world_pos(coord, origin, &overlay.params);
        let placed = Transform {
            translation: Vec3::new(pos.x, 0.5, pos.z),
            scale: Vec3::splat(overlay.params.hex_size),
            ..*transform
        };
        transform.set_if_neq(placed);
        visibility.set_if_neq(Visibility::Visible);
        hovered.set_if_neq(HoveredHex(Some(coord)));
    } else {
        visibility.set_if_neq(Visibility::Hidden);
        hovered.set_if_neq(HoveredHex(None));
    }
}

// -- Acted markers -------------------------------------------------------------

/// A status pip on a counter: what the unit has done or suffered this
/// phase or turn, the counter's own marker as on the tabletop.
#[derive(Component)]
pub struct ActedMarker;

/// What a status pip says, in drawing order along the counter's top edge.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Pip {
    /// Spent movement points this turn (§5.12).
    Moved,
    /// Fired in this fire subphase (§6.14) -- or its Maxims did (§2.32).
    Fired,
    /// Melee-attacked this turn (§7.5).
    Meleed,
    /// Stopped in an enemy zone of control: no further this turn (§5.43).
    ZocStopped,
}

impl Pip {
    fn color(self) -> Color {
        match self {
            Pip::Moved => Color::srgb(0.35, 0.55, 0.95),
            Pip::Fired => Color::srgb(1.0, 0.55, 0.1),
            Pip::Meleed => Color::srgb(0.85, 0.1, 0.1),
            Pip::ZocStopped => Color::srgb(1.0, 0.9, 0.2),
        }
    }

    /// The legend line for a pip (the hover tooltip lists them).
    pub(crate) fn meaning(self) -> &'static str {
        match self {
            Pip::Moved => "moved this turn (§5.12)",
            Pip::Fired => "fired this subphase (§6.14)",
            Pip::Meleed => "melee-attacked this turn (§7.5)",
            Pip::ZocStopped => "stopped in an enemy ZOC -- no further this turn (§5.43)",
        }
    }
}

/// The pips `unit` carries in `gs`.
pub(crate) fn pips_of(
    gs: &omdurman_rules::effects::GameState,
    unit: omdurman_rules::UnitId,
) -> Vec<Pip> {
    let mut pips = Vec::new();
    if gs.mp_spent(unit) > 0 {
        pips.push(Pip::Moved);
    }
    if gs.phase.is_fire()
        && (gs.units_fired_this_phase.contains(&unit)
            || gs.gunboat_maxims_fired_this_phase.contains(&unit))
    {
        pips.push(Pip::Fired);
    }
    if gs.units_meleed_this_turn.contains(&unit) {
        pips.push(Pip::Meleed);
    }
    if gs.zoc_stopped_this_turn.contains(&unit) {
        pips.push(Pip::ZocStopped);
    }
    pips
}

/// The pip square and one material per [`Pip`], made on first use.
type PipAssets = (Handle<Mesh>, Vec<(Pip, Handle<StandardMaterial>)>);

/// Every counter carrying pips, with them: what was last drawn.
type CounterPips = Vec<(Entity, Vec<Pip>)>;

/// Hang status pips ([`Pip`]) on the counters: small coloured squares
/// along each counter's top edge, children of the counter so they follow
/// it through the stack layout. Rebuilt only when what they show changed
/// (or the overlays were cleared, see [`crate::picker::OverlayGeneration`]).
pub fn update_acted_markers(
    mut draw: crate::overlay::Overlay,
    game_state: Res<crate::GameStateResource>,
    counters: Query<(Entity, &crate::picker::PlacedUnit)>,
    existing: Query<Entity, With<ActedMarker>>,
    overlay: Res<HexOverlay>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets: Local<Option<PipAssets>>,
    mut last: Local<Option<CounterPips>>,
) {
    if draw.stale() {
        *last = None;
    }
    let gs = &game_state.0;
    let mut marked: CounterPips = counters
        .iter()
        .filter_map(|(entity, placed)| {
            let pips = pips_of(gs, placed.unit_id?);
            (!pips.is_empty()).then_some((entity, pips))
        })
        .collect();
    marked.sort_by_key(|(e, _)| e.to_bits());
    if last.as_ref() == Some(&marked) {
        return;
    }
    let existing: Vec<Entity> = existing.iter().collect();
    draw.clear(existing.iter().copied());

    let sprite = overlay.params.hex_size * crate::picker::SPRITE_HEX_FRACTION;
    let pip = sprite * 0.16;
    let (mesh, colors) = assets.get_or_insert_with(|| {
        let mesh = meshes.add(Rectangle::new(1.0, 1.0));
        let colors = [Pip::Moved, Pip::Fired, Pip::Meleed, Pip::ZocStopped]
            .into_iter()
            .map(|p| {
                let material = materials.add(StandardMaterial {
                    base_color: p.color(),
                    unlit: true,
                    ..default()
                });
                (p, material)
            })
            .collect();
        (mesh, colors)
    });
    for (counter, pips) in &marked {
        for (i, kind) in pips.iter().enumerate() {
            let Some((_, material)) = colors.iter().find(|(p, _)| p == kind) else {
                continue;
            };
            // Along the top edge from the right corner, in the counter's own
            // plane (local +Y is the counter's top), a hair above it.
            let x = sprite * 0.5 - pip * (0.7 + 1.15 * i as f32);
            let y = sprite * 0.5 - pip * 0.7;
            draw.commands.entity(*counter).with_child((
                ActedMarker,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(x, y, 0.02).with_scale(Vec3::splat(pip)),
                Visibility::Inherited,
            ));
        }
    }
    *last = Some(marked);
}

// -- The Nile current -----------------------------------------------------------

/// Marker for a Nile current arrow.
#[derive(Component)]
pub struct NileFlowArrow;

/// While a gunboat is selected, a faint arrow on every Nile hex shows the
/// current (§5.24): a step with it is downstream, a step against it
/// upstream, on the boat's smaller upstream allowance. Built when a boat is
/// selected, cleared when it is not.
pub fn nile_flow_arrows(
    mut draw: crate::overlay::Overlay,
    picker: Res<crate::picker::PickerState>,
    placed_units: Query<(Entity, &crate::picker::PlacedUnit)>,
    game_state: Res<crate::GameStateResource>,
    existing: Query<Entity, With<NileFlowArrow>>,
    mut shown: Local<bool>,
) {
    if draw.stale() {
        *shown = false;
    }
    let gs = &game_state.0;
    let boat = crate::picker::selected_unit_id(&picker, &placed_units)
        .and_then(|(id, _)| gs.find_unit(id))
        .is_some_and(|u| u.profile.kind.is_boat());
    if boat == *shown {
        return;
    }
    draw.clear(existing.iter());
    *shown = boat;
    if !boat {
        return;
    }
    let origin = draw.hex.layout.adjusted_origin(&draw.hex.overlay.params);
    let size = draw.hex.overlay.params.hex_size;
    for (&at, _) in gs.board.terrain.iter() {
        let Some(flow) = gs.board.flow_at(at) else {
            continue;
        };
        let toward = at.neighbors()[flow as usize];
        let from = hex_world_pos(at, origin, &draw.hex.overlay.params);
        let to = hex_world_pos(toward, origin, &draw.hex.overlay.params);
        let delta = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
        let len = delta.length();
        if len < f32::EPSILON {
            continue;
        }
        let dir = delta / len;
        // A short arrow across the hex's middle, pointing down the current.
        let draw_len = len * 0.45;
        let tail = from - dir * (draw_len * 0.5);
        draw.commands.spawn((
            NileFlowArrow,
            Mesh3d(draw.arrow_assets.mesh.clone()),
            MeshMaterial3d(draw.arrow_assets.dim.clone()),
            Transform::from_xyz(tail.x, 0.6, tail.z)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                .with_scale(Vec3::new(size * 0.3, 1.0, draw_len)),
            Visibility::Visible,
        ));
    }
}

// -- Roads ---------------------------------------------------------------------

/// Runtime toggle for the road overlay, beside ZOC and LOS in the toolbar.
#[derive(Resource, Default)]
pub struct RoadOverlay {
    pub visible: bool,
}

/// Marker for one road link bar.
#[derive(Component)]
pub struct RoadBar;

/// With the Roads toggle on, a bar joins the centres of every two hexes
/// the board links by road (§5.11: a road step costs 1 MP whatever the
/// terrain) -- the board data drawn over the printed map, so the two can be
/// read together. Rebuilt when toggled or on a board load.
pub fn road_overlay(
    mut draw: crate::overlay::Overlay,
    toggle: Res<RoadOverlay>,
    game_state: Res<crate::GameStateResource>,
    existing: Query<Entity, With<RoadBar>>,
    mut shown: Local<Option<usize>>,
) {
    if draw.stale() {
        *shown = None;
    }
    let roads = &game_state.0.board.roads;
    let want = toggle.visible.then_some(roads.len());
    if *shown == want && !toggle.is_changed() {
        return;
    }
    draw.clear(existing.iter());
    *shown = want;
    if want.is_none() {
        return;
    }
    let origin = draw.origin();
    let size = draw.size();
    for road in roads {
        let a = hex_world_pos(road.a, origin, &draw.hex.overlay.params);
        let b = hex_world_pos(road.b, origin, &draw.hex.overlay.params);
        let mid = (a + b) * 0.5;
        let len = a.distance(b).max(0.001);
        let dir = (b - a) / len;
        let angle = (-dir.z).atan2(dir.x);
        draw.commands.spawn((
            RoadBar,
            Mesh3d(draw.hex.assets.unit_square.clone()),
            MeshMaterial3d(draw.hex.assets.blue.clone()),
            Transform::from_translation(Vec3::new(mid.x, 1.3, mid.z))
                .with_rotation(Quat::from_rotation_y(angle) * Quat::from_rotation_x(-PI / 2.0))
                .with_scale(Vec3::new(len, size * 0.07, 1.0)),
            Visibility::Visible,
        ));
    }
}

// -- The Mahdi's Tomb ----------------------------------------------------------

/// Marker for the ring on the Mahdi's Tomb hex.
#[derive(Component)]
pub struct MahdisTombMarker;

/// Ring the Mahdi's Tomb (§9.14: 25 victory points to whoever controls it
/// at the conclusion of play) in the colour of its holder: blue once the
/// Anglo-Egyptians have taken it, orange while the Dervish hold it. The
/// Campaign only -- the Tomb is on no other map. Rebuilt only when the
/// holder changes (or the overlays were cleared).
pub fn mahdis_tomb_marker(
    mut draw: crate::overlay::Overlay,
    game_state: Res<crate::GameStateResource>,
    existing: Query<Entity, With<MahdisTombMarker>>,
    mut last: Local<Option<Option<(HexCoord, omdurman_types::Player)>>>,
) {
    // A board loaded since: the ring was placed on the old calibration.
    if draw.stale() {
        *last = None;
    }
    if last.is_some() && !game_state.is_changed() {
        return;
    }
    let gs = &game_state.0;
    let shown = (gs.scenario == omdurman_types::Scenario::Campaign)
        .then(|| {
            let tomb = gs
                .board
                .hex_of_location(omdurman_types::Location::MahdisTomb)?;
            let holder = omdurman_rules::effects::mahdis_tomb_controller(gs)?;
            Some((tomb, holder))
        })
        .flatten();
    if last.as_ref() == Some(&shown) {
        return;
    }
    draw.clear(existing.iter());
    if let Some((tomb, holder)) = shown {
        let material = match holder {
            omdurman_types::Player::AngloEgyptian => &draw.hex.assets.blue,
            omdurman_types::Player::Dervish => &draw.hex.assets.orange,
        };
        draw.ring(MahdisTombMarker, tomb, 1.45, 1.05, material.clone());
    }
    *last = Some(shown);
}

// -- Helpers -------------------------------------------------------------------

// -- Hex ring mesh --------------------------------------------------------

/// Shared mesh + colored materials for hex ring outlines.
#[derive(Resource)]
#[cfg_attr(test, derive(Default))]
pub struct HexRingAssets {
    pub mesh: Handle<Mesh>,
    /// A unit (1×1) square quad, scaled at spawn to outline a unit counter.
    pub unit_square: Handle<Mesh>,
    pub red: Handle<StandardMaterial>,
    pub green: Handle<StandardMaterial>,
    pub light_green: Handle<StandardMaterial>,
    pub orange: Handle<StandardMaterial>,
    /// Blue outline for the selected Anglo-Egyptian unit.
    pub blue: Handle<StandardMaterial>,
    /// Bright near-white outline for the unit under the cursor (hover).
    pub hover: Handle<StandardMaterial>,
    /// Translucent fill for the cursor hex when placement is legal.
    pub marker_green: Handle<StandardMaterial>,
    /// Translucent fill for the cursor hex when placement is illegal.
    pub marker_red: Handle<StandardMaterial>,
    /// Translucent neutral fill for the cursor hex with nothing in hand.
    pub marker_idle: Handle<StandardMaterial>,
    pub gray: Handle<StandardMaterial>,
    /// Movement-range outline (see `overlay_palette::RING_REACH`).
    pub reach: Handle<StandardMaterial>,
    pub yellow: Handle<StandardMaterial>,
    pub path_shadow: Handle<StandardMaterial>,
    pub fire_arrow: Handle<StandardMaterial>,
}

fn unlit_alpha_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    }
}

pub fn spawn_hex_ring_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(hex_ring_mesh());
    let unit_square = meshes.add(Rectangle::new(1.0, 1.0));
    use overlay_palette as op;
    let red = materials.add(unlit_alpha_material(op::RING_HOSTILE));
    let green = materials.add(unlit_alpha_material(op::RING_LEGAL));
    let light_green = materials.add(unlit_alpha_material(op::RING_LEGAL_SOFT));
    let orange = materials.add(unlit_alpha_material(op::RING_ORANGE));
    let blue = materials.add(unlit_alpha_material(op::RING_BLUE));
    let hover = materials.add(unlit_alpha_material(op::RING_HOVER));
    let marker_green = materials.add(unlit_alpha_material(op::MARKER_LEGAL));
    let marker_red = materials.add(unlit_alpha_material(op::MARKER_ILLEGAL));
    let marker_idle = materials.add(unlit_alpha_material(op::MARKER_IDLE));
    let gray = materials.add(unlit_alpha_material(op::RING_NEUTRAL));
    let reach = materials.add(unlit_alpha_material(op::RING_REACH));
    let yellow = materials.add(unlit_alpha_material(op::RING_YELLOW));
    let path_shadow = materials.add(unlit_alpha_material(op::PATH_SHADOW));
    let fire_arrow = materials.add(unlit_alpha_material(op::FIRE_ARROW));
    commands.insert_resource(HexRingAssets {
        mesh,
        unit_square,
        red,
        green,
        light_green,
        orange,
        blue,
        hover,
        marker_green,
        marker_red,
        marker_idle,
        gray,
        reach,
        yellow,
        path_shadow,
        fire_arrow,
    });
}

// -- Movement-path arrow mesh ---------------------------------------------

/// A unit-length arrow lying in the XZ plane, tail at the origin and tip at
/// `(0, 0, 1)`. Built once and reused for every path segment: the spawn system
/// scales its length and rotates `+Z` onto the segment's heading (via
/// `Quat::from_rotation_arc`, matching the Nile-arrow convention), so one mesh
/// serves all arrows regardless of length or direction.
///
/// Geometry: a thin rectangular shaft from `z=0` to `z=SHAFT_END`, then a
/// triangular head from `SHAFT_END` to the tip at `z=1`. Width is along X.
fn arrow_mesh() -> Mesh {
    const SHAFT_END: f32 = 0.68;
    const SHAFT_HALF: f32 = 0.09;
    const HEAD_HALF: f32 = 0.24;

    let positions: Vec<Vec3> = vec![
        // shaft quad
        Vec3::new(-SHAFT_HALF, 0.0, 0.0),
        Vec3::new(SHAFT_HALF, 0.0, 0.0),
        Vec3::new(SHAFT_HALF, 0.0, SHAFT_END),
        Vec3::new(-SHAFT_HALF, 0.0, SHAFT_END),
        // head triangle
        Vec3::new(-HEAD_HALF, 0.0, SHAFT_END),
        Vec3::new(HEAD_HALF, 0.0, SHAFT_END),
        Vec3::new(0.0, 0.0, 1.0),
    ];
    // Wind both faces CCW when viewed from +Y (top); cull_mode is None anyway.
    let indices = vec![0u32, 2, 1, 0, 3, 2, 4, 6, 5];
    crate::fx::transient::flat_mesh(positions, indices)
}

/// Shared arrow mesh + dim/highlight materials for movement-path rendering.
#[derive(Resource)]
#[cfg_attr(test, derive(Default))]
pub struct MovementArrowAssets {
    pub mesh: Handle<Mesh>,
    /// Faint fill for paths not currently hovered.
    pub dim: Handle<StandardMaterial>,
    /// Bright fill for the path under the cursor.
    pub bright: Handle<StandardMaterial>,
    /// Fill of the combat aim / allocation arrows.
    pub combat: Handle<StandardMaterial>,
}

pub fn spawn_movement_arrow_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(arrow_mesh());
    // Mild orange: dim (faint) for idle paths, brighter for the hovered one.
    let dim = materials.add(unlit_alpha_material(overlay_palette::MOVE_ARROW));
    let bright = materials.add(unlit_alpha_material(overlay_palette::MOVE_ARROW_HOVER));
    let combat = materials.add(unlit_alpha_material(overlay_palette::COMBAT_ARROW));
    commands.insert_resource(MovementArrowAssets {
        mesh,
        dim,
        bright,
        combat,
    });
}

/// Registers all render-domain resources and systems: the map plane, hex
/// selection marker, and the counters' status pips.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HexOverlay::default())
            .insert_resource(HoveredHex::default())
            .insert_resource(HoveredUnit::default())
            .add_systems(
                Startup,
                (
                    spawn_map_plane,
                    add_plane_picking.after(spawn_map_plane),
                    spawn_selection_marker,
                    spawn_hex_ring_assets,
                ),
            )
            .add_systems(
                Update,
                (
                    update_selection_marker.run_if(crate::off_title),
                    // Board markers are drawn only on the board view of a
                    // live or reviewed game; `clear_gameplay_overlays`
                    // despawns them when that view is left.
                    (
                        update_acted_markers,
                        // §6.64 shell-burst rings on this turn's howitzer impacts.
                        crate::fire::howitzer_impact_markers,
                        // §10.11/§10.21 Dervish-secret mine & chain markers.
                        crate::river_placement::mine_chain_overlay_mesh,
                        // §10.11/§10.21: where the mine or chain may go.
                        crate::river_placement::river_placement_band,
                        // §9.14: who holds the Mahdi's Tomb.
                        mahdis_tomb_marker,
                        // §5.24: the current, while a gunboat is selected.
                        nile_flow_arrows,
                        // §5.11: the road links, on the toolbar toggle.
                        road_overlay,
                    )
                        .run_if(crate::on_board),
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The hovered hex is published with `set_if_neq`: a pointer resting on
    /// one hex leaves `HoveredHex` unchanged frame after frame, so systems
    /// gated on it (the path arrows) stay idle.
    #[test]
    fn a_resting_pointer_leaves_the_hovered_hex_unchanged() {
        #[derive(Resource, Default)]
        struct Changes(usize);
        fn count(hovered: Res<HoveredHex>, mut n: ResMut<Changes>) {
            if hovered.is_changed() {
                n.0 += 1;
            }
        }
        let mut app = App::new();
        let layout = omdurman_board_ui::board_store::default_layout();
        let overlay = HexOverlay::default();
        let hex = HexCoord::new(2, 2);
        let at = hex_world_pos(
            hex,
            layout.adjusted_origin(&overlay.params),
            &overlay.params,
        );
        let mut map = GameMap::default();
        map.hexes.insert(
            hex,
            omdurman_types::HexData {
                terrain: omdurman_types::Terrain::Clear,
                name: None,
                setup_letter: None,
                named_area: None,
            },
        );
        app.insert_resource(crate::picking::PointerGroundHit(Some(at)))
            .insert_resource(layout)
            .insert_resource(overlay)
            .insert_resource(map)
            .init_resource::<HoveredHex>()
            .init_resource::<Changes>()
            .add_systems(Update, (update_selection_marker, count).chain());
        app.world_mut()
            .spawn((Transform::default(), Visibility::Hidden, SelectionMarker));
        app.update();
        assert_eq!(app.world().resource::<HoveredHex>().0, Some(hex));
        let first = app.world().resource::<Changes>().0;
        for _ in 0..5 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<Changes>().0,
            first,
            "no change while the pointer rests"
        );
    }
}

#[cfg(test)]
mod tomb_marker_tests {
    use super::*;

    /// §9.14: the Campaign's Tomb hex is ringed for its holder.
    #[test]
    fn the_tomb_is_ringed_in_the_campaign() {
        let mut app = App::new();
        let mut gs = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        gs.board = omdurman_rules::board_data::board_info(omdurman_types::MapKind::Campaign);
        app.insert_resource(HexRingAssets::default())
            .init_resource::<MovementArrowAssets>()
            .insert_resource(omdurman_board_ui::board_store::default_layout())
            .insert_resource(HexOverlay::default())
            .insert_resource(crate::GameStateResource(gs))
            .init_resource::<crate::picker::OverlayGeneration>()
            .add_systems(Update, mahdis_tomb_marker);
        app.update();
        let rings = app
            .world_mut()
            .query::<&MahdisTombMarker>()
            .iter(app.world())
            .count();
        assert_eq!(rings, 1);
    }
}

#[cfg(test)]
mod flow_arrow_tests {
    use super::*;

    /// §5.24: selecting a gunboat shows the current on every Nile hex the
    /// board annotates; deselecting clears it.
    #[test]
    fn a_selected_gunboat_shows_the_current() {
        let mut app = App::new();
        let mut gs = omdurman_rules::effects::GameState::new(omdurman_types::Scenario::Campaign);
        gs.board = omdurman_rules::board_data::board_info(omdurman_types::MapKind::Campaign);
        let boat = omdurman_rules::UnitId::BritishBoats_3_0;
        gs.units.push(omdurman_rules::UnitPlacement {
            id: boat,
            position: HexCoord::new(34, 11),
            profile: omdurman_rules::unit_profiles::profile_for_unit(boat).unwrap(),
            state: Default::default(),
        });
        let annotated = gs
            .board
            .terrain
            .keys()
            .filter(|h| gs.board.flow_at(**h).is_some())
            .count();
        app.insert_resource(HexRingAssets::default())
            .init_resource::<MovementArrowAssets>()
            .insert_resource(MovementArrowAssets::default())
            .insert_resource(omdurman_board_ui::board_store::default_layout())
            .insert_resource(HexOverlay::default())
            .insert_resource(crate::GameStateResource(gs))
            .insert_resource(crate::picker::PickerState::Idle)
            .init_resource::<crate::picker::OverlayGeneration>()
            .add_systems(Update, nile_flow_arrows);
        let source = app
            .world_mut()
            .spawn(crate::picker::PlacedUnit {
                coord: HexCoord::new(34, 11),
                section_name: omdurman_types::SectionName::BritishBoats,
                col: 3,
                row: 0,
                is_boat: true,
                unit_id: Some(boat),
                disrupted: false,
            })
            .id();
        let count = |app: &mut App| {
            app.world_mut()
                .query::<&NileFlowArrow>()
                .iter(app.world())
                .count()
        };
        app.update();
        assert_eq!(count(&mut app), 0, "nothing selected");
        *app.world_mut().resource_mut::<crate::picker::PickerState>() =
            crate::picker::PickerState::Selected {
                source,
                start_coord: HexCoord::new(34, 11),
                remaining_mp: 0,
                forced_stop: false,
            };
        app.update();
        assert!(annotated > 0);
        assert_eq!(count(&mut app), annotated);
        *app.world_mut().resource_mut::<crate::picker::PickerState>() =
            crate::picker::PickerState::Idle;
        app.update();
        assert_eq!(count(&mut app), 0);
    }
}

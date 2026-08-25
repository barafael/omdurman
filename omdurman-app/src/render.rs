use std::f32::consts::PI;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use omdurman_hexmap::{GameMap, HexLayout};

use omdurman_hexmap::{hex_ring_mesh, hex_world_pos, hit_to_hex};

// Re-exported for the rest of the app (the definitions live in
// `omdurman-hexmap::plane`, shared with the map editor tool).
pub use omdurman_hexmap::{
    HexOverlay, MapPlane, MapTextureCache, PlaneTextureStores, apply_map_data_to_plane,
};

use crate::{camera::RtsCamera, util::raycast_ground};
use omdurman_types::HexCoord;

// -- Render resources -------------------------------------------------------

/// Written every frame by `render::update_selection_marker` with the hex
/// currently under the cursor (or `None` if no valid hex is hovered).
#[derive(Resource, Default)]
pub struct HoveredHex(pub Option<HexCoord>);

/// The placed-unit entity currently under the cursor (the specific counter in
/// a stack, resolved by `update_hovered_unit`), or `None`. Drives the bright
/// hover square that previews which unit a click would select.
#[derive(Resource, Default)]
pub struct HoveredUnit(pub Option<bevy::prelude::Entity>);

// -- Map plane -----------------------------------------------------------------

pub fn spawn_map_plane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loaded: Res<crate::LoadedAnnotations>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Startup spawns the default Fall-of-Khartoum board; the plane is re-sized
    // and re-textured by `apply_map_data_to_plane` when a scenario selects a
    // board (§dual-map).
    //
    // Preload *both* boards' textures now so their (slow, ~30 MB) decode +
    // GPU upload overlaps the lobby/menu instead of stalling the first board
    // switch. The decode runs off the main thread; switching boards later then
    // just swaps to an already-resident handle (see `MapTextureCache`).
    let mut cache = MapTextureCache::default();
    for kind in [
        omdurman_types::MapKind::FallOfKhartoum,
        omdurman_types::MapKind::Campaign,
    ] {
        cache.texture(&asset_server, &loaded.map(kind).image);
    }
    let texture = cache.texture(&asset_server, "fall_of_khartoum_1885.webp");
    commands.insert_resource(cache);
    commands.spawn((
        MapPlane,
        Name::new("MapPlane"),
        Mesh3d(meshes.add(Rectangle::new(
            omdurman_hexmap::IMG_W,
            omdurman_hexmap::IMG_H,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
    ));
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
        base_color: Color::srgba(1.0, 0.0, 0.0, 0.25),
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
/// records the hovered hex coordinate in [`HoveredHex`] for the UI.
pub fn update_selection_marker(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<SelectionMarker>>,
    mut hovered: ResMut<HoveredHex>,
) {
    let Ok((mut transform, mut visibility)) = marker.single_mut() else {
        hovered.0 = None;
        return;
    };
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        *visibility = Visibility::Hidden;
        hovered.0 = None;
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    if game_map.hexes.contains_key(&coord) {
        let pos = hex_world_pos(coord, origin, &overlay.params);
        transform.translation = Vec3::new(pos.x, 0.5, pos.z);
        transform.scale = Vec3::splat(overlay.params.hex_size);
        *visibility = Visibility::Visible;
        hovered.0 = Some(coord);
    } else {
        *visibility = Visibility::Hidden;
        hovered.0 = None;
    }
}

// -- Acted marker ---------------------------------------------------------------

/// Translucent grey-blue ring drawn on top of a unit's hex to indicate it
/// has already acted (fired, moved, etc.) during the current phase.
#[derive(Component)]
pub struct ActedMarker;

/// Spawn or despawn acted-outline rings for every unit that has spent
/// movement points this phase.  Runs every frame (despawn-all + respawn)
/// like the fire/melee target rings.
///
/// TODO(acted-universal): extend to fire allocations and melee once the
/// rules engine has a universal `acted` field on `UnitState`.
pub fn update_acted_markers(
    mut commands: Commands,
    hex: crate::HexRender,
    game_state: Option<Res<crate::GameStateResource>>,
    existing: Query<Entity, With<ActedMarker>>,
) {
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);

    let Some(gs) = game_state else { return };
    let origin = hex.layout.adjusted_origin(&hex.overlay.params);
    let size = hex.overlay.params.hex_size;

    for unit in gs.0.units.iter().filter(|u| u.state.disrupted || gs.0.mp_spent(u.id) > 0) {
        let pos = hex_world_pos(unit.position, origin, &hex.overlay.params);
        commands.spawn((
            ActedMarker,
            Mesh3d(hex.assets.mesh.clone()),
            MeshMaterial3d(hex.assets.acted.clone()),
            Transform::from_xyz(pos.x, 0.8, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
}

// -- Helpers -------------------------------------------------------------------

// -- Hex ring mesh --------------------------------------------------------

/// Shared mesh + colored materials for hex ring outlines.
#[derive(Resource)]
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
    /// Translucent fill for the cursor hex when placement is illegal / idle.
    pub marker_red: Handle<StandardMaterial>,
    pub gray: Handle<StandardMaterial>,
    pub yellow: Handle<StandardMaterial>,
    pub path_shadow: Handle<StandardMaterial>,
    pub fire_arrow: Handle<StandardMaterial>,
    /// Grey-blue for the per-unit "acted" ring.
    pub acted: Handle<StandardMaterial>,
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
    let red = materials.add(unlit_alpha_material(Color::srgb(1.0, 0.0, 0.0)));
    let green = materials.add(unlit_alpha_material(Color::srgb(0.0, 1.0, 0.0)));
    let light_green = materials.add(unlit_alpha_material(Color::srgb(0.6, 1.0, 0.6)));
    let orange = materials.add(unlit_alpha_material(Color::srgb(1.0, 0.55, 0.1)));
    let blue = materials.add(unlit_alpha_material(Color::srgb(0.25, 0.55, 1.0)));
    let hover = materials.add(unlit_alpha_material(Color::srgb(1.0, 0.97, 0.55)));
    let marker_green = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 1.0, 0.0, 0.25),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let marker_red = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.0, 0.0, 0.25),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let gray = materials.add(unlit_alpha_material(Color::srgb(0.4, 0.4, 0.4)));
    let yellow = materials.add(unlit_alpha_material(Color::srgba(1.0, 0.85, 0.0, 0.4)));
    let path_shadow =
        materials.add(unlit_alpha_material(Color::srgba(0.45, 0.55, 0.95, 0.18)));
    let fire_arrow =
        materials.add(unlit_alpha_material(Color::srgba(0.9, 0.15, 0.1, 0.45)));
    // Grey-blue for "acted" outline — slightly transparent so the unit
    // counter underneath is still visible.
    let acted = materials.add(unlit_alpha_material(Color::srgba(0.45, 0.60, 0.80, 0.35)));
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
        gray,
        yellow,
        path_shadow,
        fire_arrow,
        acted,
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
    let n = positions.len();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::Y; n])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; n])
}

/// Shared arrow mesh + dim/highlight materials for movement-path rendering.
#[derive(Resource)]
pub struct MovementArrowAssets {
    pub mesh: Handle<Mesh>,
    /// Faint fill for paths not currently hovered.
    pub dim: Handle<StandardMaterial>,
    /// Bright fill for the path under the cursor.
    pub bright: Handle<StandardMaterial>,
}

pub fn spawn_movement_arrow_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(arrow_mesh());
    // Mild orange: dim (faint) for idle paths, brighter for the hovered one.
    let dim = materials.add(unlit_alpha_material(Color::srgba(0.85, 0.5, 0.2, 0.45)));
    let bright = materials.add(unlit_alpha_material(Color::srgba(0.95, 0.55, 0.2, 0.95)));
    commands.insert_resource(MovementArrowAssets { mesh, dim, bright });
}

/// Registers all render-domain resources and systems: the map plane, hex
/// selection marker, and the acted markers.
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
                    spawn_selection_marker,
                    spawn_hex_ring_assets,
                ),
            )
            .add_systems(
                Update,
                (
                    update_selection_marker.run_if(crate::hex_hover_visible),
                    update_acted_markers,
                ),
            );
    }
}

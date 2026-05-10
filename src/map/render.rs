use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use crate::RtsCamera;
use crate::map::HexCoord;
use crate::map::layout::{MAP_W, MAP_H, HexLayout};

// ── Map plane ─────────────────────────────────────────────────────────────────

/// Marker so other systems can query the map plane entity.
#[derive(Component)]
pub struct MapPlane;

/// Startup system — spawns the flat textured plane that shows the map image.
///
/// The plane lies on the XZ ground (Y = 0), centred at the world origin.
/// Game entities are spawned at Y > 0 so they render on top.
pub fn spawn_map_plane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let texture: Handle<Image> = asset_server.load("fall_of_khartoum_1885.png");

    commands.spawn((
        MapPlane,
        Name::new("MapPlane"),
        // Rectangle primitive lives in the XY plane; rotate -90° around X
        // so it lies flat on the XZ ground plane.
        Mesh3d(meshes.add(Rectangle::new(MAP_W, MAP_H))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
    ));
}

// ── Hex debug overlay ─────────────────────────────────────────────────────────

/// A handful of hexes rendered in red so we can visually confirm that the
/// calibrated grid lines up with the printed hexes on the map image.
const EXAMPLE_HEXES: &[(i32, i32)] = &[
    ( 0,  0),   // Austrian Mission — calibration ref
    ( 1,  0),   // one step east
    ( 0,  1),   // one step south-east
    (-1,  1),   // one step south-west
    ( 5, -1),   // Barracks — second calibration ref
];

/// Text node that shows the axial coordinate of the hex under the cursor.
#[derive(Component)]
pub struct HoverCoordText;

pub fn spawn_hover_coord_text(mut commands: Commands) {
    commands.spawn((
        HoverCoordText,
        Text::new(""),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(1.0, 0.1, 0.1)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(14.0),
            left: Val::Px(18.0),
            ..default()
        },
    ));
}

pub fn draw_hex_debug(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    mut gizmos: Gizmos,
    mut text_q: Query<&mut Text, With<HoverCoordText>>,
) {
    let red = Color::srgb(1.0, 0.0, 0.0);

    // Static example hexes.
    for &(q, r) in EXAMPLE_HEXES {
        let center = layout.hex_to_world(HexCoord::new(q, r));
        draw_hex_outline(&mut gizmos, center, layout.hex_size, red);
    }

    // Hover: ray from camera → Y=0 plane → hex coord.
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else { return };

    let dir = ray.direction.as_vec3();
    if dir.y.abs() < 1e-6 { return; }
    let t = -ray.origin.y / dir.y;
    if t < 0.0 { return; }
    let hit = ray.origin + dir * t;

    let coord = layout.world_to_hex(hit);
    let center = layout.hex_to_world(coord);
    draw_hex_outline(&mut gizmos, center, layout.hex_size, red);

    if let Ok(mut text) = text_q.single_mut() {
        *text = Text::new(format!("hex  q {}  r {}", coord.q, coord.r));
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

/// Draw a flat-top hex outline on the XZ plane via gizmos.
fn draw_hex_outline(gizmos: &mut Gizmos, center: Vec3, size: f32, color: Color) {
    // Flat-top vertices at angles 0°, 60°, 120°, 180°, 240°, 300°.
    let verts: [Vec3; 6] = std::array::from_fn(|k| {
        let angle = k as f32 * PI / 3.0;
        Vec3::new(
            center.x + size * angle.cos(),
            1.0,   // just above the map plane
            center.z + size * angle.sin(),
        )
    });
    for i in 0..6 {
        gizmos.line(verts[i], verts[(i + 1) % 6], color);
    }
}

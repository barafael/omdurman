//! Road connection bars: pooled brown mesh quads drawn on the ground plane
//! between hex centres for each road edge in the game map.

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout, SQRT_3, hex_world_pos};
use omdurman_types::Orientation;

use omdurman_hexmap::HexOverlay;

use super::{EditorToolState, hexside::place_hexside_quad};

/// SystemParam bundling mutable access to the road quad pool, commands,
/// materials, and the per-entity query — mirrors [`HexsideQuadPool`].
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct RoadQuadPool<'w, 's> {
    quads: ResMut<'w, RoadQuads>,
    commands: Commands<'w, 's>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    q: Query<
        'w,
        's,
        (
            &'static mut Transform,
            &'static mut Visibility,
            &'static MeshMaterial3d<StandardMaterial>,
        ),
        With<RoadQuad>,
    >,
}

/// A pooled road bar (a flat quad on the ground plane).
#[derive(Component)]
pub struct RoadQuad;

/// Reusable pool of road-bar entities + the shared unit-square mesh.
#[derive(Resource, Default)]
pub struct RoadQuads {
    mesh: Handle<Mesh>,
    pool: Vec<Entity>,
}

/// One-time setup: the shared unit quad mesh for road bars.
pub fn setup_road_quads(mut quads: ResMut<RoadQuads>, mut meshes: ResMut<Assets<Mesh>>) {
    quads.mesh = meshes.add(Rectangle::new(1.0, 1.0));
}

/// Road bar width as a fraction of hex size -- chunky enough to be obvious.
const ROAD_WIDTH_FRAC: f32 = 0.10;

/// How far a road extends from a non-crossroad hex's center toward the edge,
/// as a fraction of the centre-to-edge distance. 0.75 means the road stops
/// 25 % in from the edge, making it visibly enter the tile without reaching
/// the centre (which is what the crossroad flag does).
const ROAD_END_FRAC: f32 = 0.75;

/// Intersection of the ray from `center` toward `target` with the boundary of
/// a regular hexagon of circumradius `size` and the given `orientation`. The
/// returned point lies on the hex edge between two vertices.
fn hex_edge_intersection(center: Vec3, size: f32, orientation: Orientation, target: Vec3) -> Vec3 {
    let dx = target.x - center.x;
    let dz = target.z - center.z;
    let len = (dx * dx + dz * dz).sqrt();
    if len < 0.0001 {
        return center;
    }
    let (ndx, ndz) = (dx / len, dz / len);

    let apothem = size * SQRT_3 / 2.0;

    let normals: [(f32, f32); 6] = match orientation {
        Orientation::Pointy => [
            (1.0, 0.0),
            (0.5, SQRT_3 * 0.5),
            (-0.5, SQRT_3 * 0.5),
            (-1.0, 0.0),
            (-0.5, -SQRT_3 * 0.5),
            (0.5, -SQRT_3 * 0.5),
        ],
        Orientation::Flat => [
            (SQRT_3 * 0.5, -0.5),
            (SQRT_3 * 0.5, 0.5),
            (0.0, 1.0),
            (-SQRT_3 * 0.5, 0.5),
            (-SQRT_3 * 0.5, -0.5),
            (0.0, -1.0),
        ],
    };

    let mut min_t = f32::MAX;
    for &(nx, nz) in &normals {
        let dot = ndx * nx + ndz * nz;
        if dot > 0.0 {
            let t = apothem / dot;
            if t < min_t {
                min_t = t;
            }
        }
    }

    Vec3::new(center.x + ndx * min_t, center.y, center.z + ndz * min_t)
}

/// Place a brown road bar for every road edge in the game map. Pool grows on
/// demand; unused bars are parked invisible.
pub(super) fn update_road_quads(
    mode: EditorToolState,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut pool: RoadQuadPool,
) {
    if !mode.is_editor() {
        for &entity in &pool.quads.pool {
            if let Ok((_, mut visibility, _)) = pool.q.get_mut(entity) {
                *visibility = Visibility::Hidden;
            }
        }
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let base_w = overlay.params.hex_size * ROAD_WIDTH_FRAC;
    let color = Color::srgb(0.5, 0.3, 0.1);

    let edges: Vec<(Vec3, Vec3)> = game_map
        .roads
        .iter()
        .map(|edge| {
            let a_pos = hex_world_pos(edge.a, origin, &overlay.params);
            let b_pos = hex_world_pos(edge.b, origin, &overlay.params);

            let a_is_crossroad = game_map
                .hexes
                .get(&edge.a)
                .map(|d| d.terrain.is_crossroad())
                .unwrap_or(false);
            let b_is_crossroad = game_map
                .hexes
                .get(&edge.b)
                .map(|d| d.terrain.is_crossroad())
                .unwrap_or(false);

            let p0 = if a_is_crossroad {
                a_pos
            } else {
                let edge = hex_edge_intersection(
                    a_pos,
                    overlay.params.hex_size,
                    overlay.params.orientation,
                    b_pos,
                );
                a_pos + (edge - a_pos) * ROAD_END_FRAC
            };
            let p1 = if b_is_crossroad {
                b_pos
            } else {
                let edge = hex_edge_intersection(
                    b_pos,
                    overlay.params.hex_size,
                    overlay.params.orientation,
                    a_pos,
                );
                b_pos + (edge - b_pos) * ROAD_END_FRAC
            };

            (Vec3::new(p0.x, 1.3, p0.z), Vec3::new(p1.x, 1.3, p1.z))
        })
        .collect();

    while pool.quads.pool.len() < edges.len() {
        let material = pool.materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let id = pool
            .commands
            .spawn((
                RoadQuad,
                Mesh3d(pool.quads.mesh.clone()),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        pool.quads.pool.push(id);
    }

    for (i, &entity) in pool.quads.pool.iter().enumerate() {
        let Ok((mut transform, mut visibility, mat_handle)) = pool.q.get_mut(entity) else {
            continue;
        };
        if let Some(&(p0, p1)) = edges.get(i) {
            if let Some(mut material) = pool.materials.get_mut(&mat_handle.0) {
                place_hexside_quad(&mut transform, &mut material, p0, p1, base_w, 1.3, color);
            }
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

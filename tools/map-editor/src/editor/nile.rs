//! Nile-current flow-direction arrows: pooled orange arrow meshes placed on
//! Nile hexes during terrain editing. Shown only in the terrain editor tab.

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use omdurman_hexmap::{GameMap, HexLayout, hex_world_pos};
use omdurman_types::HexCoord;

use omdurman_hexmap::HexOverlay;

use super::EditorToolState;

/// A pooled Nile-current arrow mesh entity.
#[derive(Component)]
pub struct NileArrow;

/// Reusable pool of Nile-arrow entities + the shared arrow mesh/material.
#[derive(Resource, Default)]
pub struct NileArrows {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    pool: Vec<Entity>,
}

/// Build a flat arrow mesh pointing +Z, centred at origin, length ~~ 1.
fn make_arrow_mesh() -> Mesh {
    let sw = 0.04;
    let hw = 0.14;
    let positions = vec![
        Vec3::new(-sw, 0.0, -0.4),
        Vec3::new(sw, 0.0, -0.4),
        Vec3::new(sw, 0.0, 0.2),
        Vec3::new(-sw, 0.0, 0.2),
        Vec3::new(-hw, 0.0, 0.2),
        Vec3::new(hw, 0.0, 0.2),
        Vec3::new(0.0, 0.0, 0.6),
    ];
    let normals = vec![Vec3::Y; 7];
    let indices = Indices::U32(vec![0, 2, 1, 0, 3, 2, 4, 6, 5]);
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(indices)
}

/// One-time setup: the shared arrow mesh/material.
pub fn setup_nile_arrows(
    mut arrows: ResMut<NileArrows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    arrows.mesh = meshes.add(make_arrow_mesh());
    arrows.material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.55, 0.0),
        unlit: true,
        ..default()
    });
}

/// Arrow length as a fraction of hex size.
const NILE_ARROW_LEN_FRAC: f32 = 0.7;

/// Place one orange flow-direction arrow per Nile hex that has a current
/// annotation; shown only in the terrain editor. Pool grows on demand; unused
/// arrows are parked invisible.
pub(super) fn update_nile_arrows(
    mode: EditorToolState,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut arrows: ResMut<NileArrows>,
    mut commands: Commands,
    mut q: Query<(&mut Transform, &mut Visibility), With<NileArrow>>,
) {
    let active = mode.is_editor();

    let mut placements: Vec<(Vec3, Vec3)> = Vec::new();
    if active {
        let origin = layout.adjusted_origin(&overlay.params);
        for (coord, data) in &game_map.hexes {
            let Some(direction) = data.terrain.nile_direction() else {
                continue;
            };
            let Some(dir) = flow_world_dir(*coord, direction, origin, &overlay.params) else {
                continue;
            };
            let center = hex_world_pos(*coord, origin, &overlay.params);
            placements.push((Vec3::new(center.x, 1.5, center.z), dir));
        }
    }

    while arrows.pool.len() < placements.len() {
        let id = commands
            .spawn((
                NileArrow,
                Mesh3d(arrows.mesh.clone()),
                MeshMaterial3d(arrows.material.clone()),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        arrows.pool.push(id);
    }

    let scale = overlay.params.hex_size * NILE_ARROW_LEN_FRAC;
    for (i, &entity) in arrows.pool.iter().enumerate() {
        let Ok((mut transform, mut visibility)) = q.get_mut(entity) else {
            continue;
        };
        if let Some(&(center, dir)) = placements.get(i) {
            *transform = Transform::from_translation(center)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                .with_scale(Vec3::splat(scale));
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Direction in the ground plane (XZ) the Nile current flows for a hex with
/// the given `direction`, derived from the hex's world centre and the centre of
/// its `direction`-th neighbour so it stays correct under any orientation / stagger.
/// `None` when the neighbour and hex coincide (degenerate overlay).
fn flow_world_dir(
    coord: HexCoord,
    direction: omdurman_types::HexDirection,
    origin: bevy::math::Vec2,
    overlay: &omdurman_types::OverlayParams,
) -> Option<Vec3> {
    let c = hex_world_pos(coord, origin, overlay);
    let n = hex_world_pos(coord.neighbors()[direction as usize], origin, overlay);
    let v = Vec3::new(n.x - c.x, 0.0, n.z - c.z);
    let len = v.length();
    (len > 1e-3).then(|| v / len)
}

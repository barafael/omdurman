//! Hexside bar rendering: pooled flat coloured quads laid on the map for each
//! painted hexside, plus hover-preview and selected-segment highlights in the
//! hexside editor tab.

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use omdurman_hexmap::{hex_world_pos, hit_to_hex};
use omdurman_types::{HexsideKind, HexsideRef};

use crate::render::HexOverlay;
use crate::util::raycast_ground;

use super::{EditorToolState, HexEditor, HexSpatial, nearest_edge};

/// The endpoints of the short bar drawn along the shared border of `edge`
/// (the perpendicular-bisector segment at the midpoint of the two hex centres).
pub(super) fn hexside_segment(edge: &HexsideRef, origin: Vec2, overlay: &HexOverlay) -> (Vec3, Vec3) {
    let a = hex_world_pos(edge.a, origin, &overlay.params);
    let b = hex_world_pos(edge.b, origin, &overlay.params);
    let mid = (a + b) * 0.5;
    let along = (b - a).normalize_or_zero();
    let perp = Vec3::new(-along.z, 0.0, along.x);
    let half = overlay.params.hex_size * 0.5;
    (
        Vec3::new(mid.x, 1.0, mid.z) - perp * half,
        Vec3::new(mid.x, 1.0, mid.z) + perp * half,
    )
}

/// A pooled hexside bar (a flat quad on the ground plane).
#[derive(Component)]
pub struct HexsideQuad;

/// Reusable pool of hexside quad entities + the shared unit-square mesh they all
/// use (scaled per-bar via Transform). Materials are per-entity so each bar can
/// take its own colour.
#[derive(Resource, Default)]
pub struct HexsideQuads {
    mesh: Handle<Mesh>,
    pool: Vec<Entity>,
}

/// One-time setup: create the shared unit quad mesh used by every hexside bar.
pub fn setup_hexside_quads(mut quads: ResMut<HexsideQuads>, mut meshes: ResMut<Assets<Mesh>>) {
    quads.mesh = meshes.add(Rectangle::new(1.0, 1.0));
}

/// Bar width as a fraction of hex size -- chunky enough to be obvious.
const HEXSIDE_WIDTH_FRAC: f32 = 0.16;

/// Place a flat coloured quad over the hexside `(p0, p1)` segment: centred on
/// the segment, rotated to lie along it, scaled to (length x width). `width`
/// and `y` (height above the map) and `color` are caller-chosen so selection /
/// hover bars can be wider, higher, and brighter than plain ones.
pub(super) fn place_hexside_quad(
    transform: &mut Transform,
    material: &mut StandardMaterial,
    p0: Vec3,
    p1: Vec3,
    width: f32,
    y: f32,
    color: Color,
) {
    let mid = (p0 + p1) * 0.5;
    let len = p0.distance(p1).max(0.001);
    let dir = (p1 - p0) / len;
    let angle = (-dir.z).atan2(dir.x);
    *transform = Transform::from_translation(Vec3::new(mid.x, y, mid.z))
        .with_rotation(
            Quat::from_rotation_y(angle) * Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
        )
        .with_scale(Vec3::new(len, width, 1.0));
    material.base_color = color;
}

/// Mutable pool/asset state for the hexside-quad rebuild: the quad pool, the
/// material store, the command buffer, and the per-entity query. Bundled so
/// [`update_hexside_quads`] stays under the system-parameter limit.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct HexsideQuadPool<'w, 's> {
    quads: ResMut<'w, HexsideQuads>,
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
        With<HexsideQuad>,
    >,
}

/// Rebuild the hexside quad pool each frame from the painted hexsides plus the
/// hover/selection bars, in the terrain Editor and Hexside editor modes.
/// Unused pooled quads are parked invisible.
pub(super) fn update_hexside_quads(
    mode: EditorToolState,
    spatial: HexSpatial,
    editor: Res<HexEditor>,
    mut contexts: EguiContexts,
    mut pool: HexsideQuadPool,
) {
    if !mode.is_hexside()
        && !mode.is_changed()
        && !spatial.overlay.is_changed()
        && !spatial.game_map.is_changed()
        && !editor.is_changed()
    {
        return;
    }

    let active = mode.is_editor() || mode.is_hexside();

    let mut bars: Vec<(Vec3, Vec3, f32, f32, Color)> = Vec::new();
    if active {
        let origin = spatial.layout.adjusted_origin(&spatial.overlay.params);
        let base_w = spatial.overlay.params.hex_size * HEXSIDE_WIDTH_FRAC;
        for (edge, kind) in &spatial.game_map.hexsides {
            let (p0, p1) = hexside_segment(edge, origin, &spatial.overlay);
            bars.push((p0, p1, base_w, 1.2, hexside_color(*kind)));
        }
        if mode.is_hexside() {
            let over_ui = contexts
                .ctx_mut()
                .map(|c| c.egui_wants_pointer_input())
                .unwrap_or(false);
            if !over_ui && let Some(hit) = raycast_ground(&spatial.windows, &spatial.cameras) {
                let coord = hit_to_hex(hit, origin, &spatial.overlay.params);
                if spatial.game_map.hexes.contains_key(&coord)
                    && let Some(edge) = nearest_edge(coord, hit, origin, &spatial.overlay.params)
                    && editor.selected_hexside != Some(edge)
                {
                    let (p0, p1) = hexside_segment(&edge, origin, &spatial.overlay);
                    bars.push((p0, p1, base_w * 1.6, 1.4, Color::srgba(0.2, 0.9, 1.0, 0.6)));
                }
            }
            if let Some(edge) = editor.selected_hexside {
                let (p0, p1) = hexside_segment(&edge, origin, &spatial.overlay);
                bars.push((p0, p1, base_w * 1.9, 1.6, Color::srgb(0.2, 0.9, 1.0)));
            }
        }
    }

    while pool.quads.pool.len() < bars.len() {
        let material = pool.materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let id = pool
            .commands
            .spawn((
                HexsideQuad,
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
        if let Some(&(p0, p1, width, y, color)) = bars.get(i) {
            if let Some(mut material) = pool.materials.get_mut(&mat_handle.0) {
                place_hexside_quad(&mut transform, &mut material, p0, p1, width, y, color);
            }
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub(super) fn hexside_color(kind: HexsideKind) -> Color {
    match kind {
        HexsideKind::Wall => Color::srgb(0.75, 0.75, 0.75),
        HexsideKind::Gate => Color::srgb(0.9, 0.8, 0.2),
        HexsideKind::Breach => Color::srgb(0.9, 0.4, 0.1),
        HexsideKind::Khor => Color::srgb(0.4, 0.3, 0.15),
        HexsideKind::Crest => Color::srgb(0.6, 0.45, 0.3),
        HexsideKind::ZaribaThornHedge => Color::srgb(0.3, 0.55, 0.2),
        HexsideKind::ZaribaTrench => Color::srgb(0.5, 0.5, 0.6),
        HexsideKind::ZaribaTrenchEndA => Color::srgb(0.6, 0.6, 0.7),
        HexsideKind::ZaribaTrenchEndB => Color::srgb(0.6, 0.6, 0.7),
        HexsideKind::KhorShambat => Color::srgb(0.2, 0.45, 0.55),
    }
}

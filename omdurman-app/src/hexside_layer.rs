//! Dynamic hexside rendering (§5.3, §6.62/§6.63): pooled flat coloured bars
//! laid on the map for every hexside that *changed* since the board was
//! loaded — artillery/engineer wall breaches and campaign-constructed
//! zariba hexsides. The authored map texture already shows the printed
//! walls/khors, so only the engine's mutations are drawn here.
//!
//! The drawing is ported from the map editor's hexside tab
//! (`tools/map-editor/src/editor/hexside.rs`): bars along the
//! perpendicular-bisector segment of the two hex centres, with the same
//! per-kind colours.

use bevy::prelude::*;
use omdurman_hexmap::{HexLayout, HexOverlay, hex_world_pos};
use omdurman_types::{HexsideKind, HexsideRef};

use crate::GameStateResource;

/// A pooled hexside bar (a flat quad on the ground plane).
#[derive(Component)]
struct HexsideBar;

/// Reusable pool of hexside bar entities + the shared unit-square mesh.
#[derive(Resource, Default)]
struct HexsideBars {
    mesh: Option<Handle<Mesh>>,
    pool: Vec<Entity>,
}

/// Bar width as a fraction of hex size — chunky enough to be obvious.
const HEXSIDE_WIDTH_FRAC: f32 = 0.16;
/// Height above the map plane (above unit counters' base plane, below the
/// floating hex rings).
const HEXSIDE_Y: f32 = 1.2;

/// The endpoints of the short bar drawn along the shared border of `edge`
/// (the perpendicular-bisector segment at the midpoint of the two hex centres).
fn hexside_segment(
    edge: &HexsideRef,
    origin: bevy::math::Vec2,
    overlay: &HexOverlay,
) -> (Vec3, Vec3) {
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

/// The same per-kind colours the map editor paints with, so an in-game breach
/// looks exactly like the editor's authoring preview.
fn hexside_color(kind: HexsideKind) -> Color {
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

/// Place a flat coloured quad over the hexside `(p0, p1)` segment: centred on
/// the segment, rotated to lie along it, scaled to (length × width).
fn place_hexside_quad(
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

/// Rebuild the hexside bar pool from the hexsides whose engine kind differs
/// from the authored map (breaches, constructed zariba). Runs only when the
/// game state changed. Unused pooled quads are parked invisible.
#[allow(clippy::too_many_arguments)]
fn update_dynamic_hexside_bars(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_state: Option<Res<GameStateResource>>,
    game_map: Option<Res<omdurman_hexmap::GameMap>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    mut pool: ResMut<HexsideBars>,
    mut bars_q: Query<
        (
            Entity,
            &mut Transform,
            &mut Visibility,
            &MeshMaterial3d<StandardMaterial>,
        ),
        With<HexsideBar>,
    >,
) {
    let Some(gs) = game_state else { return };
    if !gs.is_changed() {
        return;
    }
    let Some(game_map) = game_map else { return };

    let mut bars: Vec<(Vec3, Vec3, f32, Color)> = Vec::new();
    for (edge, kind) in &gs.0.board.hexsides {
        if game_map.hexsides.get(edge) == Some(kind) {
            continue; // unchanged — the authored map texture already shows it
        }
        let (p0, p1) = hexside_segment(edge, layout.adjusted_origin(&overlay.params), &overlay);
        bars.push((
            p0,
            p1,
            overlay.params.hex_size * HEXSIDE_WIDTH_FRAC,
            hexside_color(*kind),
        ));
    }

    let mesh = pool
        .mesh
        .get_or_insert_with(|| meshes.add(Rectangle::new(1.0, 1.0)))
        .clone();

    // Grow the pool to fit.
    while pool.pool.len() < bars.len() {
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let id = commands
            .spawn((
                HexsideBar,
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::Hidden,
            ))
            .id();
        pool.pool.push(id);
    }

    for (i, entity) in pool.pool.iter().enumerate() {
        let Ok((_, mut transform, mut visibility, mat_handle)) = bars_q.get_mut(*entity) else {
            continue;
        };
        if let Some(&(p0, p1, width, color)) = bars.get(i) {
            if let Some(mut material) = materials.get_mut(&mat_handle.0) {
                place_hexside_quad(
                    &mut transform,
                    &mut material,
                    p0,
                    p1,
                    width,
                    HEXSIDE_Y,
                    color,
                );
            }
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Registers the dynamic hexside layer.
pub struct HexsideLayerPlugin;

impl Plugin for HexsideLayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HexsideBars>()
            .add_systems(Update, update_dynamic_hexside_bars);
    }
}

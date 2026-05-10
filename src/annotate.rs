use bevy::prelude::*;

use crate::map::layout::{world_to_pixel, HexLayout};
use crate::map::HexCoord;

// ── Marker component for spawned dots ──────────────────────────────────────────

#[derive(Component)]
pub struct AnnotationDot;

// ── Resource ───────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct AnnotationSession {
    pub active: bool,
    pub points: Vec<AnnotationPoint>,
    pub dot_entities: Vec<Entity>,
}

#[derive(Clone, Debug)]
pub struct AnnotationPoint {
    pub hex: HexCoord,
    pub pixel: Vec2,
}

// ── Systems ────────────────────────────────────────────────────────────────────

pub fn toggle_annotation_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut session: ResMut<AnnotationSession>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }
    session.active = !session.active;
    if session.active {
        info!("Annotation mode ACTIVE — click hexes to record them. Tab to exit.");
    } else {
        for &entity in &session.dot_entities {
            commands.entity(entity).despawn();
        }
        session.dot_entities.clear();
        print_results(&session);
        session.points.clear();
    }
}

pub fn handle_annotation_click(
    buttons: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut session: ResMut<AnnotationSession>,
    layout: Res<HexLayout>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if !session.active || !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    if ray.direction.y.abs() < f32::EPSILON {
        return;
    }
    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return;
    }
    let world_pos = ray.origin + ray.direction * t;

    let hex = layout.world_to_hex(world_pos);
    let pixel = world_to_pixel(world_pos);

    info!("Annotated: ({:>3}, {:>3})  px ({:>8.0}, {:>8.0})", hex.q, hex.r, pixel.x, pixel.y);

    session.points.push(AnnotationPoint { hex, pixel });

    let entity = commands.spawn((
        AnnotationDot,
        Mesh3d(meshes.add(Sphere::new(5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(world_pos),
    )).id();
    session.dot_entities.push(entity);
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn print_results(session: &AnnotationSession) {
    info!("═══════════════════════════════════════════════════");
    info!("  Annotation Results");
    info!("  {} point(s)", session.points.len());
    info!("───────────────────────────────────────────────────");
    info!("  {:<14} {:<12} {:<12}", "Hex (q, r)", "Pixel X", "Pixel Y");
    info!("  ─────────────────────────────────────────────────");
    for pt in &session.points {
        info!("  ({:>3}, {:>3})    {:>8.0}    {:>8.0}", pt.hex.q, pt.hex.r, pt.pixel.x, pt.pixel.y);
    }
    info!("═══════════════════════════════════════════════════");
}

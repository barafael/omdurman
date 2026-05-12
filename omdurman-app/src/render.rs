use std::f32::consts::{FRAC_PI_6, PI};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hex::HexLayout;
use omdurman_map::GameMap;

use crate::RtsCamera;
use crate::util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground};

// ── Map plane ─────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct MapPlane;

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
        Mesh3d(meshes.add(Rectangle::new(omdurman_hex::MAP_W, omdurman_hex::MAP_H))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
    ));
}

// ── Hex overlay resource ──────────────────────────────────────────────────────

/// Ctrl+1 — adjustable hex grid overlay for layout calibration.
#[derive(Resource)]
pub struct HexOverlay {
    pub visible: bool,
    pub hex_size: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for HexOverlay {
    fn default() -> Self {
        Self {
            visible: false,
            hex_size: 51.0,
            offset_x: -1.0,
            offset_y: 1.0,
        }
    }
}

// ── Egui overlay panel ────────────────────────────────────────────────────────

pub fn overlay_ui(mut contexts: EguiContexts, mut overlay: ResMut<HexOverlay>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !overlay.visible {
        return;
    }
    egui::Window::new("overlay")
        .default_pos([14.0, 14.0])
        .resizable(false)
        .title_bar(true)
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            ui.horizontal(|ui| {
                ui.label("size");
                ui.add(
                    egui::DragValue::new(&mut overlay.hex_size)
                        .speed(0.5)
                        .range(1.0..=200.0)
                        .clamp_existing_to_range(true),
                );
            });
            ui.horizontal(|ui| {
                ui.label("x");
                ui.add(
                    egui::DragValue::new(&mut overlay.offset_x)
                        .speed(1.0)
                        .clamp_existing_to_range(false),
                );
            });
            ui.horizontal(|ui| {
                ui.label("y");
                ui.add(
                    egui::DragValue::new(&mut overlay.offset_y)
                        .speed(1.0)
                        .clamp_existing_to_range(false),
                );
            });
        });
}

// ── Overlay adjustment keys (U/Y/I/K/J/L) ────────────────────────────────────

pub fn hex_overlay_adjust(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut overlay: ResMut<HexOverlay>,
) {
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
        return;
    }
    let size_step = 0.5;
    if keys.just_pressed(KeyCode::KeyU) {
        overlay.hex_size += size_step;
    }
    if keys.just_pressed(KeyCode::KeyY) {
        overlay.hex_size = (overlay.hex_size - size_step).max(1.0);
    }
    let offset_step = 5.0;
    if keys.just_pressed(KeyCode::KeyI) {
        overlay.offset_y -= offset_step;
    }
    if keys.just_pressed(KeyCode::KeyK) {
        overlay.offset_y += offset_step;
    }
    if keys.just_pressed(KeyCode::KeyJ) {
        overlay.offset_x -= offset_step;
    }
    if keys.just_pressed(KeyCode::KeyL) {
        overlay.offset_x += offset_step;
    }
}

// ── Selection marker ──────────────────────────────────────────────────────────

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

/// Moves a translucent hex marker to whichever map hex the cursor is over.
pub fn update_selection_marker(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    viewer: Res<crate::units::UnitViewer>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<SelectionMarker>>,
) {
    if viewer.visible {
        if let Ok((_, mut visibility)) = marker.single_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }
    let Ok((mut transform, mut visibility)) = marker.single_mut() else {
        return;
    };
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        *visibility = Visibility::Hidden;
        return;
    };
    let origin = adjusted_origin(&layout, overlay.offset_x, overlay.offset_y);
    let coord = hit_to_hex(hit, origin, overlay.hex_size);

    if game_map.hexes.contains_key(&coord) {
        let pos = hex_world_pos(coord, origin, overlay.hex_size);
        transform.translation = Vec3::new(pos.x, 0.5, pos.z);
        transform.scale = Vec3::splat(overlay.hex_size);
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

// ── Hex grid outlines (overlay mode only) ────────────────────────────────────

pub fn draw_hex_debug(
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut gizmos: Gizmos,
) {
    if !overlay.visible {
        return;
    }
    let origin = adjusted_origin(&layout, overlay.offset_x, overlay.offset_y);

    for coord in game_map.hexes.keys() {
        let pos = hex_world_pos(*coord, origin, overlay.hex_size);
        draw_hex_outline(
            &mut gizmos,
            pos,
            overlay.hex_size,
            Color::srgb(1.0, 0.0, 0.0),
        );
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

pub fn draw_hex_outline(gizmos: &mut Gizmos, center: Vec3, size: f32, color: Color) {
    let verts: [Vec3; 6] = std::array::from_fn(|k| {
        let angle = FRAC_PI_6 + k as f32 * PI / 3.0;
        Vec3::new(
            center.x + size * angle.cos(),
            1.0,
            center.z + size * angle.sin(),
        )
    });
    for i in 0..6 {
        gizmos.line(verts[i], verts[(i + 1) % 6], color);
    }
}

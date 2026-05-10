use std::f32::consts::{PI, FRAC_PI_6};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::RtsCamera;
use crate::map::layout::{cube_round, MAP_W, MAP_H, SQRT_3, HexLayout};

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
        Mesh3d(meshes.add(Rectangle::new(MAP_W, MAP_H))),
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

pub fn overlay_ui(
    mut contexts: EguiContexts,
    mut overlay: ResMut<HexOverlay>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !overlay.visible {
        return;
    }

    egui::Window::new("overlay")
        .default_pos([14.0, 14.0])
        .resizable(false)
        .title_bar(true)
        .show(ctx, |ui| {
            ui.style_mut().override_font_id =
                Some(egui::FontId::monospace(13.0));

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

// ── Overlay keyboard controls (suppressed when egui wants keyboard) ───────────

pub fn hex_overlay_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut overlay: ResMut<HexOverlay>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        if ctx.wants_keyboard_input() {
            return;
        }
    }

    if keys.just_pressed(KeyCode::Digit1) && keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        overlay.visible = !overlay.visible;
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

// ── Hover coord text ──────────────────────────────────────────────────────────

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

// ── Hex grid drawing (pointy-top) ─────────────────────────────────────────────

pub fn draw_hex_debug(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    mut gizmos: Gizmos,
    mut coord_text_q: Query<&mut Text, With<HoverCoordText>>,
) {
    // Always use the tuned overlay parameters (size 51, offset -1, 1).
    let ox = layout.origin.x + overlay.offset_x;
    let oy = layout.origin.y + overlay.offset_y;
    let hs = overlay.hex_size;

    // Full grid overlay (only when toggled on via Ctrl+1)
    if overlay.visible {
        let half_w = MAP_W / 2.0;
        let half_h = MAP_H / 2.0;

        let q_min = ((-half_w - ox) / (SQRT_3 * hs) - (half_h - oy) / (3.0 * hs)).floor() as i32 - 2;
        let q_max = ((half_w - ox) / (SQRT_3 * hs) - (-half_h - oy) / (3.0 * hs)).ceil() as i32 + 2;
        let r_min = ((-half_h - oy) / (1.5 * hs)).floor() as i32 - 2;
        let r_max = ((half_h - oy) / (1.5 * hs)).ceil() as i32 + 2;

        for q in q_min..=q_max {
            for r in r_min..=r_max {
                let cx = ox + hs * SQRT_3 * (q as f32 + r as f32 * 0.5);
                let cz = oy + hs * 1.5 * r as f32;
                draw_hex_outline(&mut gizmos, Vec3::new(cx, 0.0, cz), hs, Color::srgb(1.0, 0.0, 0.0));
            }
        }
    }

    // Hover (always active — uses the same overlay parameters)
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else { return };

    let dir = ray.direction.as_vec3();
    if dir.y.abs() < 1e-6 { return; }
    let t = -ray.origin.y / dir.y;
    if t < 0.0 { return; }
    let hit = ray.origin + dir * t;

    let coord = {
        let dx = hit.x - ox;
        let dz = hit.z - oy;
        let fq = (dx * SQRT_3 / 3.0 - dz / 3.0) / hs;
        let fr = (dz * 2.0 / 3.0) / hs;
        cube_round(fq, fr)
    };
    let chx = ox + hs * SQRT_3 * (coord.q as f32 + coord.r as f32 * 0.5);
    let chz = oy + hs * 1.5 * coord.r as f32;
    draw_hex_outline(&mut gizmos, Vec3::new(chx, 0.0, chz), hs, Color::srgb(1.0, 0.0, 0.0));

    if let Ok(mut text) = coord_text_q.single_mut() {
        *text = Text::new(format!("hex  q {}  r {}", coord.q, coord.r));
    }
}

// ── Helper ────────────────────────────────────────────────────────────────────

pub(crate) fn draw_hex_outline(gizmos: &mut Gizmos, center: Vec3, size: f32, color: Color) {
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

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hex::{HexLayout, SQRT_3, cube_round};
use omdurman_map::{GameMap, MapInfoPath, save_game_map};
use omdurman_types::{HexCoord, HexData, Terrain};

use crate::RtsCamera;
use crate::render::{HexOverlay, draw_hex_outline};

#[derive(Resource)]
pub struct HexEditor {
    pub active: bool,
    pub selected: Option<HexCoord>,
    pub name: String,
    pub terrain: Terrain,
}

impl Default for HexEditor {
    fn default() -> Self {
        Self {
            active: false,
            selected: None,
            name: String::new(),
            terrain: Terrain::Desert,
        }
    }
}

fn hex_center(coord: HexCoord, layout: &HexLayout, overlay: &HexOverlay) -> Vec3 {
    let ox = layout.origin.x + overlay.offset_x;
    let oy = layout.origin.y + overlay.offset_y;
    let hs = overlay.hex_size;
    Vec3::new(
        ox + hs * SQRT_3 * (coord.q as f32 + coord.r as f32 * 0.5),
        0.1,
        oy + hs * 1.5 * coord.r as f32,
    )
}

/// Ctrl+2 toggles the hex editor; B/C/D/F/P/S/V/W set terrain on the selected hex.
pub fn editor_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut editor: ResMut<HexEditor>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        if ctx.wants_keyboard_input() {
            return;
        }
    }
    if keys.just_pressed(KeyCode::Digit2)
        && keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
        editor.active = !editor.active;
        if !editor.active {
            editor.selected = None;
        }
        return;
    }
    if editor.selected.is_none() {
        return;
    }
    let t = if keys.just_pressed(KeyCode::KeyB) {
        Some(Terrain::BlueNile)
    } else if keys.just_pressed(KeyCode::KeyC) {
        Some(Terrain::City)
    } else if keys.just_pressed(KeyCode::KeyD) {
        Some(Terrain::Desert)
    } else if keys.just_pressed(KeyCode::KeyF) {
        Some(Terrain::Fortress)
    } else if keys.just_pressed(KeyCode::KeyP) {
        Some(Terrain::Palm)
    } else if keys.just_pressed(KeyCode::KeyS) {
        Some(Terrain::Shrubs)
    } else if keys.just_pressed(KeyCode::KeyV) {
        Some(Terrain::Village)
    } else if keys.just_pressed(KeyCode::KeyW) {
        Some(Terrain::WhiteNile)
    } else {
        None
    };
    if let Some(t) = t {
        editor.terrain = t;
    }
}

pub fn handle_hex_editor_click(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut editor: ResMut<HexEditor>,
) {
    if !editor.active || !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut() {
        if ctx.wants_pointer_input() {
            return;
        }
    }
    let Ok(window) = windows.single() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else {
        return;
    };
    let dir = ray.direction.as_vec3();
    if dir.y.abs() < 1e-6 {
        return;
    }
    let t = -ray.origin.y / dir.y;
    if t < 0.0 {
        return;
    }
    let hit = ray.origin + dir * t;
    let ox = layout.origin.x + overlay.offset_x;
    let oy = layout.origin.y + overlay.offset_y;
    let hs = overlay.hex_size;
    let dx = hit.x - ox;
    let dz = hit.z - oy;
    let fq = (dx * SQRT_3 / 3.0 - dz / 3.0) / hs;
    let fr = (dz * 2.0 / 3.0) / hs;
    let coord = cube_round(fq, fr);

    if game_map.hexes.contains_key(&coord) {
        editor.selected = Some(coord);
        let data = game_map.hexes.get(&coord);
        editor.name = data.and_then(|d| d.name.clone()).unwrap_or_default();
        editor.terrain = data.map(|d| d.terrain).unwrap_or_default();
    }
}

pub fn draw_editor_highlight(
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    editor: Res<HexEditor>,
    mut gizmos: Gizmos,
) {
    if !editor.active {
        return;
    }
    let Some(coord) = editor.selected else { return };
    let ox = layout.origin.x + overlay.offset_x;
    let oy = layout.origin.y + overlay.offset_y;
    let hs = overlay.hex_size;
    let cx = ox + hs * SQRT_3 * (coord.q as f32 + coord.r as f32 * 0.5);
    let cz = oy + hs * 1.5 * coord.r as f32;
    draw_hex_outline(
        &mut gizmos,
        Vec3::new(cx, 0.0, cz),
        hs,
        Color::srgb(0.0, 1.0, 0.0),
    );
}

pub fn editor_labels_ui(
    mut contexts: EguiContexts,
    editor: Res<HexEditor>,
    game_map: Res<GameMap>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
) {
    if !editor.active {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(vp_size) = camera.logical_viewport_size() else {
        return;
    };
    for (coord, data) in &game_map.hexes {
        let pos = hex_center(*coord, &layout, &overlay);
        let Ok(screen) = camera.world_to_viewport(cam_transform, pos) else {
            continue;
        };
        if screen.x < 0.0 || screen.x > vp_size.x || screen.y < 0.0 || screen.y > vp_size.y {
            continue;
        }
        let text = match &data.name {
            Some(n) => format!("{}\n{}", data.terrain, n),
            None => format!("{}", data.terrain),
        };
        let num_lines = text.lines().count() as f32;
        let max_line = text.lines().map(str::len).max().unwrap_or(0) as f32;
        let text_w = max_line * 6.0;
        let text_h = num_lines * 14.0;
        egui::Area::new(egui::Id::new(("hl", coord.q, coord.r)))
            .fixed_pos(egui::pos2(screen.x - text_w / 2.0, screen.y - text_h / 2.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.style_mut().override_font_id = Some(egui::FontId::monospace(10.0));
                ui.colored_label(egui::Color32::BLACK, text);
            });
    }
}

pub fn editor_ui(
    mut contexts: EguiContexts,
    mut editor: ResMut<HexEditor>,
    mut game_map: ResMut<GameMap>,
    map_info_path: Res<MapInfoPath>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !editor.active {
        return;
    }
    egui::Window::new("hex editor")
        .default_pos([14.0, 200.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            if let Some(coord) = editor.selected {
                ui.label(format!("hex  q {}  r {}", coord.q, coord.r));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("name");
                    ui.add(egui::TextEdit::singleline(&mut editor.name).desired_width(120.0));
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label("type");
                    egui::ComboBox::from_id_salt("terrain")
                        .selected_text(format!("{}", editor.terrain))
                        .show_ui(ui, |ui| {
                            for &t in Terrain::variants() {
                                ui.selectable_value(&mut editor.terrain, t, format!("{}", t));
                            }
                        });
                });
            } else {
                ui.label("click a hex to select");
            }
        });
    if let Some(coord) = editor.selected {
        let name = if editor.name.is_empty() {
            None
        } else {
            Some(editor.name.clone())
        };
        let terrain = editor.terrain;
        let changed = match game_map.hexes.get(&coord) {
            Some(d) => d.terrain != terrain || d.name != name,
            None => true,
        };
        if changed {
            game_map.hexes.insert(
                coord,
                HexData {
                    terrain,
                    location: None,
                    name,
                },
            );
            save_game_map(&game_map, &map_info_path.0);
        }
    }
}

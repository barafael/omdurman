use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, save_game_map};
use omdurman_types::{HexCoord, HexData, IntoEnumIterator, Terrain};

use omdurman_net::NetMsg;

use crate::util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground};
use crate::render::{HexOverlay, draw_hex_outline};
use crate::RtsCamera;
use crate::PendingEdits;

const MAP_INFO_SAVE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/map_info.ron");

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

fn hex_label_pos(coord: HexCoord, layout: &HexLayout, overlay: &HexOverlay) -> Vec3 {
    let origin = adjusted_origin(layout, overlay.offset_x, overlay.offset_y);
    let mut pos = hex_world_pos(coord, origin, overlay.hex_size);
    pos.y = 0.1;
    pos
}

/// B/C/D/F/P/S/V/W set terrain on the selected hex.
pub fn editor_terrain_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut editor: ResMut<HexEditor>,
) {
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
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
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_pointer_input()
    {
        return;
    }
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.offset_x, overlay.offset_y);
    let coord = hit_to_hex(hit, origin, overlay.hex_size);

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
    let origin = adjusted_origin(&layout, overlay.offset_x, overlay.offset_y);
    let pos = hex_world_pos(coord, origin, overlay.hex_size);
    draw_hex_outline(
        &mut gizmos,
        pos,
        overlay.hex_size,
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
        let pos = hex_label_pos(*coord, &layout, &overlay);
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
    mut pending: ResMut<PendingEdits>,
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
                            for t in Terrain::iter() {
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
            let name_str = editor.name.clone();
            let terrain_idx = Terrain::iter().position(|t| t == terrain).unwrap_or(0) as u8;
            pending.0.push(NetMsg::MapEdit {
                q: coord.q,
                r: coord.r,
                terrain: terrain_idx,
                name: name_str.clone(),
            });
            game_map.hexes.insert(
                coord,
                HexData {
                    terrain,
                    location: None,
                    name: if name_str.is_empty() {
                        None
                    } else {
                        Some(name_str)
                    },
                },
            );
            save_game_map(&game_map, MAP_INFO_SAVE_PATH);
        }
    }
}

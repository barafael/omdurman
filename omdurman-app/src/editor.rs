use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, save_annotations_to_file};
use omdurman_types::{HexCoord, HexData, IntoEnumIterator, Terrain};

use omdurman_net::NetMsg;

use crate::{
    PendingEdits, RtsCamera, SidebarClip,
    browser::SpriteAnnotationsResource,
    render::{HexOverlay, draw_hex_outline},
    util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground},
};

const ANNOTATIONS_SAVE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/annotations.ron");

#[derive(Resource, Default)]
pub struct HexEditor {
    pub active: bool,
    pub selected: Option<HexCoord>,
    pub name: String,
    pub terrain: Terrain,
}

fn hex_label_pos(coord: HexCoord, layout: &HexLayout, overlay: &HexOverlay) -> Vec3 {
    let origin = adjusted_origin(layout, overlay.params.offset_x, overlay.params.offset_y);
    let mut pos = hex_world_pos(coord, origin, &overlay.params);
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
    } else if keys.just_pressed(KeyCode::KeyD) {
        Some(Terrain::Desert)
    } else if keys.just_pressed(KeyCode::KeyF) {
        Some(Terrain::Fortress)
    } else if keys.just_pressed(KeyCode::KeyP) {
        Some(Terrain::Palm)
    } else if keys.just_pressed(KeyCode::KeyS) {
        Some(Terrain::Shrubs)
    } else if keys.just_pressed(KeyCode::KeyW) {
        Some(Terrain::WhiteNile)
    } else if keys.just_pressed(KeyCode::KeyK) {
        Some(Terrain::Khartoum)
    } else if keys.just_pressed(KeyCode::KeyT) {
        Some(Terrain::Tuti)
    } else if keys.just_pressed(KeyCode::KeyH) {
        Some(Terrain::Hogali)
    } else if keys.just_pressed(KeyCode::KeyU) {
        Some(Terrain::Buri)
    } else if keys.just_pressed(KeyCode::KeyM) {
        Some(Terrain::FortMakran)
    } else if keys.just_pressed(KeyCode::Digit1) {
        Some(Terrain::FortBuri)
    } else if keys.just_pressed(KeyCode::KeyN) {
        Some(Terrain::NorthFort)
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
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    if let Some(data) = game_map.hexes.get(&coord) {
        editor.selected = Some(coord);
        editor.name = data.name.clone().unwrap_or_default();
        editor.terrain = data.terrain;
    } else if editor.selected == Some(coord) {
        editor.selected = None;
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
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let pos = hex_world_pos(coord, origin, &overlay.params);
    draw_hex_outline(
        &mut gizmos,
        pos,
        overlay.params.hex_size,
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
    clip: Res<SidebarClip>,
) {
    if !editor.active {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else { return };
    let Some(vp_size) = camera.logical_viewport_size() else { return };

    let viewport_rect =
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(vp_size.x, vp_size.y));
    let clip_rect = if let Some(sidebar) = clip.right_sidebar {
        egui::Rect::from_min_max(
            viewport_rect.min,
            egui::pos2(sidebar.left(), viewport_rect.max.y),
        )
    } else {
        viewport_rect
    };

    let painter = ctx
        .layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("hex_labels"),
        ))
        .with_clip_rect(clip_rect);

    let font_size = 10.0;
    let char_w = font_size * 0.6;
    let line_h = font_size * 1.4;
    let padding = 3.0;
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
        let lines: Vec<&str> = text.lines().collect();
        let max_line = lines.iter().map(|l| l.len()).max().unwrap_or(0) as f32;
        let rect = egui::Rect::from_center_size(
            egui::pos2(screen.x, screen.y),
            egui::vec2(max_line * char_w + 2.0 * padding, lines.len() as f32 * line_h + 2.0 * padding),
        );
        painter.rect_filled(rect, 3.0, egui::Color32::from_black_alpha(160));
        painter.text(
            egui::pos2(screen.x, screen.y),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::monospace(font_size),
            egui::Color32::WHITE,
        );
    }
}

pub fn editor_ui(
    mut contexts: EguiContexts,
    mut editor: ResMut<HexEditor>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut clip: ResMut<SidebarClip>,
    annotations: Option<Res<SpriteAnnotationsResource>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !editor.active {
        clip.right_sidebar = None;
        return;
    }
    let response = egui::SidePanel::right("editor_panel")
        .resizable(true)
        .default_width(200.0)
        .width_range(150.0..=500.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(12, 12)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            if let Some(coord) = editor.selected {
                ui.label(format!("hex  q {}  r {}", coord.q, coord.r));
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("name");
                    ui.add(
                        egui::TextEdit::singleline(&mut editor.name).desired_width(f32::INFINITY),
                    );
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
    clip.right_sidebar = Some(response.response.rect);
    if let Some(coord) = editor.selected
        && game_map.hexes.contains_key(&coord)
    {
        let name = if editor.name.is_empty() {
            None
        } else {
            Some(editor.name.clone())
        };
        let terrain = editor.terrain;
        let changed = match game_map.hexes.get(&coord) {
            Some(d) => d.terrain != terrain || d.name != name,
            None => false,
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
            if let Some(ref ann) = annotations {
                save_annotations_to_file(&game_map, &ann.0, ANNOTATIONS_SAVE_PATH);
            }
        }
    }
}

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, save_annotations_to_file};
use omdurman_types::{HexCoord, IntoEnumIterator, Terrain};

use omdurman_net::NetMsg;

use crate::{
    EditorMode, PendingEdits, SidebarClip,
    browser::SpriteAnnotationsResource,
    camera::RtsCamera,
    render::{HexOverlay, draw_hex_outline},
    util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground},
};

pub const ANNOTATIONS_SAVE_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/annotations.ron");

#[derive(Resource, Default)]
pub struct HexEditor {
    pub selected: Option<HexCoord>,
    pub name: String,
    pub terrain: Terrain,
    pub show_terrain_overlay: bool,
}

/// B/C/D/F/P/S/V/W set terrain on the selected hex.
pub fn editor_terrain_keys(
    mode: Res<EditorMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut editor: ResMut<HexEditor>,
) {
    if *mode != EditorMode::Editor {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
        return;
    }
    if editor.selected.is_none() {
        return;
    }
    let t = match () {
        _ if keys.just_pressed(KeyCode::KeyB) => Some(Terrain::BlueNile),
        _ if keys.just_pressed(KeyCode::KeyD) => Some(Terrain::Desert),
        _ if keys.just_pressed(KeyCode::KeyF) => Some(Terrain::Fortress),
        _ if keys.just_pressed(KeyCode::KeyP) => Some(Terrain::Palm),
        _ if keys.just_pressed(KeyCode::KeyS) => Some(Terrain::Shrubs),
        _ if keys.just_pressed(KeyCode::KeyW) => Some(Terrain::WhiteNile),
        _ if keys.just_pressed(KeyCode::KeyK) => Some(Terrain::Khartoum),
        _ if keys.just_pressed(KeyCode::KeyT) => Some(Terrain::Tuti),
        _ if keys.just_pressed(KeyCode::KeyH) => Some(Terrain::Hogali),
        _ if keys.just_pressed(KeyCode::KeyU) => Some(Terrain::Buri),
        _ if keys.just_pressed(KeyCode::KeyM) => Some(Terrain::FortMakran),
        _ if keys.just_pressed(KeyCode::Digit1) => Some(Terrain::FortBuri),
        _ if keys.just_pressed(KeyCode::KeyN) => Some(Terrain::NorthFort),
        _ => None,
    };
    if let Some(t) = t {
        editor.terrain = t;
    }
}

pub fn handle_hex_editor_click(
    mode: Res<EditorMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut editor: ResMut<HexEditor>,
) {
    if *mode != EditorMode::Editor || !buttons.just_pressed(MouseButton::Left) {
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
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    editor: Res<HexEditor>,
    mut gizmos: Gizmos,
) {
    if *mode != EditorMode::Editor {
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

pub fn editor_ui(
    mut contexts: EguiContexts,
    mode: Res<EditorMode>,
    mut editor: ResMut<HexEditor>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut clip: ResMut<SidebarClip>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    annotations: Option<Res<SpriteAnnotationsResource>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if *mode != EditorMode::Editor {
        clip.right_sidebar = None;
        return;
    }

    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(vp_size) = camera.logical_viewport_size() else {
        return;
    };

    // hex labels & optional terrain colour overlay (single pass over hexes)
    {
        // Clip to the canvas area, excluding the sidebar from the previous frame so
        // background-order painters don't bleed over the panel.
        let canvas_rect = {
            let screen = ctx.viewport_rect();
            match clip.right_sidebar {
                Some(sidebar) => {
                    egui::Rect::from_min_max(screen.min, egui::pos2(sidebar.left(), screen.max.y))
                }
                None => screen,
            }
        };
        // Paint into the shared background layer so shapes append in call-order with
        // panels that share LayerId::background() (CentralPanel, SidePanel). The
        // SidePanel adds its shapes later, so they paint on top — which is what we want.
        let mut label_painter = ctx.layer_painter(egui::LayerId::background());
        label_painter.set_clip_rect(canvas_rect);
        let font_size = 10.0;
        let char_w = font_size * 0.6;
        let line_h = font_size * 1.4;
        let padding = 3.0;
        let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
        let size = overlay.params.hex_size;
        let overlay_painter = editor.show_terrain_overlay.then(|| {
            let mut p = ctx.layer_painter(egui::LayerId::background());
            p.set_clip_rect(canvas_rect);
            p
        });
        // First pass: terrain colour overlays (so labels paint on top of them).
        if let Some(ref overlay_painter) = overlay_painter {
            for (coord, data) in &game_map.hexes {
                let center = hex_world_pos(*coord, origin, &overlay.params);
                let corners = crate::render::hex_corners(Vec3::new(center.x, 1.5, center.z), size);
                let mut screen_verts = Vec::with_capacity(6);
                for world in corners {
                    if let Ok(screen) = camera.world_to_viewport(cam_transform, world) {
                        screen_verts.push(egui::pos2(screen.x, screen.y));
                    }
                }
                if screen_verts.len() == 6 {
                    let [r, g, b, a] = data.terrain.overlay_color();
                    let color = egui::Color32::from_rgba_unmultiplied(
                        (r * 255.0) as u8,
                        (g * 255.0) as u8,
                        (b * 255.0) as u8,
                        (a * 255.0) as u8,
                    );
                    overlay_painter.add(egui::Shape::convex_polygon(
                        screen_verts,
                        color,
                        egui::Stroke::NONE,
                    ));
                }
            }
        }
        // Second pass: hex labels on top of the overlay.
        for (coord, data) in &game_map.hexes {
            let center = hex_world_pos(*coord, origin, &overlay.params);
            let pos = Vec3::new(center.x, 0.1, center.z);
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
                egui::vec2(
                    max_line * char_w + 2.0 * padding,
                    lines.len() as f32 * line_h + 2.0 * padding,
                ),
            );
            label_painter.rect_filled(rect, 3.0, egui::Color32::from_black_alpha(160));
            label_painter.text(
                egui::pos2(screen.x, screen.y),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::monospace(font_size),
                egui::Color32::WHITE,
            );
        }
    }

    // ---- sidebar panel (Order::Middle, on top of background) ----
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
            ui.add_space(8.0);
            {
                let prev = editor.show_terrain_overlay;
                ui.checkbox(&mut editor.show_terrain_overlay, "terrain overlay");
                if prev != editor.show_terrain_overlay {
                    pending
                        .outgoing_broadcast
                        .push(NetMsg::ShowTerrainOverlay(editor.show_terrain_overlay));
                }
            }
        });
    clip.right_sidebar = Some(response.response.rect);
    if let Some(coord) = editor.selected
        && game_map.hexes.contains_key(&coord)
    {
        let terrain = editor.terrain;
        let editor_name = editor.name.clone();
        if let Some(d) = game_map.hexes.get_mut(&coord) {
            let new_name = (!editor_name.is_empty()).then(|| editor_name.clone());
            let changed = d.terrain != terrain || d.name != new_name;
            if changed {
                pending.outgoing_broadcast.push(NetMsg::MapEdit {
                    q: coord.q,
                    r: coord.r,
                    terrain: terrain.to_u8(),
                    name: editor_name,
                });
                d.terrain = terrain;
                d.name = new_name;
                // Map edits mutate in-memory state and are recorded in the
                // event log. Persist them back to annotations.ron so the game
                // acts as an editor for the full map.
                if let Some(ref ann) = annotations {
                    save_annotations_to_file(&game_map, &ann.0, ANNOTATIONS_SAVE_PATH);
                }
            }
        }
    }
}

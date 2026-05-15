use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, save_game_map};
use omdurman_types::{HexCoord, HexData, IntoEnumIterator, Terrain};

use omdurman_net::NetMsg;

use crate::{
    PendingEdits, RtsCamera, SidebarClip,
    render::{HexOverlay, draw_hex_outline},
    util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground},
};

const MAP_INFO_SAVE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/map_info.ron");

#[derive(Resource, Default)]
pub struct HexEditor {
    pub active: bool,
    pub selected: Option<HexCoord>,
    pub name: String,
    pub terrain: Terrain,
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

    editor.selected = Some(coord);
    let data = game_map.hexes.get(&coord);
    editor.name = data.and_then(|d| d.name.clone()).unwrap_or_default();
    editor.terrain = data.map(|d| d.terrain).unwrap_or_default();
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

#[derive(Component)]
pub struct HexLabel {
    pub coord: HexCoord,
    pub offset: Vec2,
}

fn label_offset(text: &str, font_size: f32) -> Vec2 {
    let char_w = font_size * 0.55;
    let line_h = font_size * 1.4;
    let longest = text.lines().map(|l| l.len()).max().unwrap_or(0) as f32;
    let n = text.lines().count() as f32;
    Vec2::new(-(longest * char_w) / 2.0, -(n * line_h) / 2.0)
}

pub fn editor_labels_bevy(
    mut commands: Commands,
    editor: Res<HexEditor>,
    game_map: Res<GameMap>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    mut labels: Query<(Entity, &mut Text, &mut Node, &mut HexLabel)>,
    mut was_active: Local<bool>,
) {
    let just_activated = editor.active && !*was_active;
    let just_deactivated = !editor.active && *was_active;
    *was_active = editor.active;

    if just_deactivated {
        for (entity, ..) in &labels {
            commands.entity(entity).despawn();
        }
        return;
    }

    let font_size = 10.0;

    if just_activated {
        for (coord, data) in &game_map.hexes {
            let text = match &data.name {
                Some(n) => format!("{}\n{}", data.terrain, n),
                None => format!("{}", data.terrain),
            };
            let offset = label_offset(&text, font_size);
            commands.spawn((
                Text::new(text),
                TextFont {
                    font_size,
                    ..default()
                },
                TextColor(Color::BLACK),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                HexLabel { coord: *coord, offset },
            ));
        }
    }

    if !editor.active {
        return;
    }

    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(vp_size) = camera.logical_viewport_size() else {
        return;
    };

    let map_changed = game_map.is_changed();
    for (_, mut text, mut node, mut label) in &mut labels {
        let Some(data) = game_map.hexes.get(&label.coord) else {
            continue;
        };

        if map_changed {
            let new = match &data.name {
                Some(n) => format!("{}\n{}", data.terrain, n),
                None => format!("{}", data.terrain),
            };
            *text = Text::new(&new);
            label.offset = label_offset(&new, font_size);
        }

        let pos = hex_label_pos(label.coord, &layout, &overlay);
        if let Ok(screen) = camera.world_to_viewport(cam_transform, pos) {
            if screen.x >= 0.0
                && screen.x <= vp_size.x
                && screen.y >= 0.0
                && screen.y <= vp_size.y
            {
                node.left = Val::Px(screen.x + label.offset.x);
                node.top = Val::Px(screen.y + label.offset.y);
            } else {
                node.left = Val::Px(-1000.0);
                node.top = Val::Px(-1000.0);
            }
        }
    }
}

pub fn editor_ui(
    mut contexts: EguiContexts,
    mut editor: ResMut<HexEditor>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut clip: ResMut<SidebarClip>,
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

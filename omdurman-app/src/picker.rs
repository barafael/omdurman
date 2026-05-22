use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hex::HexLayout;
use omdurman_map::GameMap;
use omdurman_types::{HexCoord, Terrain};

use crate::browser::SpriteAnnotationsResource;
use crate::camera::RtsCamera;
use crate::render::{HexOverlay, draw_hex_outline};
use crate::util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground};
use crate::{EditorMode, PendingEdits};
use omdurman_net::{GameEvent, NetMsg};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/sprites.rs"));
}

fn terrain_passable(terrain: Terrain, is_boat: bool) -> bool {
    if is_boat {
        terrain.is_nile()
    } else {
        terrain.passable_by_land()
    }
}

/// Whether a unit may occupy `coord`. Off-map coordinates (those not present
/// in `game_map.hexes`, which is clipped to the active overlay) are never
/// valid — earlier code allowed land units to be placed off-map because the
/// map wasn't guaranteed loaded; the late-joiner snapshot flow now guarantees
/// `LoadAnnotations` arrives before any placement is possible.
fn coord_passable(game_map: &GameMap, coord: HexCoord, is_boat: bool) -> bool {
    game_map
        .hexes
        .get(&coord)
        .is_some_and(|h| terrain_passable(h.terrain, is_boat))
}

// ── Resources ──────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct UnitPicker {
    pub available: Vec<PickerUnit>,
    pub all: Vec<(String, u32, u32, Handle<Image>, bool)>,
}

impl UnitPicker {
    pub fn reset_available(&mut self) {
        self.available = self
            .all
            .iter()
            .map(|(sn, col, row, handle, is_boat)| PickerUnit {
                section_name: sn.clone(),
                col: *col,
                row: *row,
                handle: handle.clone(),
                is_boat: *is_boat,
                visible: true,
                egui_texture: None,
                annotations_loaded: false,
            })
            .collect();
    }
}

pub struct PickerUnit {
    pub section_name: String,
    pub col: u32,
    pub row: u32,
    pub handle: Handle<Image>,
    pub is_boat: bool,
    pub visible: bool,
    pub egui_texture: Option<egui::TextureHandle>,
    pub annotations_loaded: bool,
}

#[derive(Resource, Default, Clone, Copy)]
pub enum PickerState {
    #[default]
    Idle,
    Placing {
        unit_idx: usize,
        preview_hex: Option<HexCoord>,
        preview_valid: bool,
        drag_drop: bool,
    },
    Moving {
        source: Entity,
        start_coord: HexCoord,
    },
}

// ── Components ─────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct PlacedUnit {
    pub coord: HexCoord,
    pub section_name: String,
    pub col: u32,
    pub row: u32,
    pub is_boat: bool,
}

#[derive(Component)]
pub struct MovementAnimation {
    pub from: Vec3,
    pub to: Vec3,
    pub progress: f32,
    pub target_coord: HexCoord,
}

// ── Startup: load sprite handles for the picker ───────────────────────────────

pub fn spawn_picker_assets(mut picker: ResMut<UnitPicker>, asset_server: Res<AssetServer>) {
    let section_order: &[&str] = &[
        "Taiasha",
        "upper_green",
        "Khalifa_Abdullah",
        "Sherif",
        "lower_green",
        "upper_Jaalin",
        "Hadendowa",
        "lower_Jaalin",
        "Hadendowa_Guns",
        "Baggara",
        "British_Boats",
        "Ali_Wad_Helu",
        "British_Army",
        "Sheik_El_Din",
        "Kitchener",
        "Jehadia",
        "Egyptian_Army",
    ];

    let mut section_sprites: Vec<Vec<PickerUnit>> =
        section_order.iter().map(|_| Vec::new()).collect();

    for &(filename, col, row) in generated::SPRITE_PATHS {
        let section_idx = section_order.iter().position(|s| {
            filename.starts_with(s) && filename.as_bytes().get(s.len()) == Some(&b'_')
        });
        if let Some(idx) = section_idx {
            let path = format!("sprites/{}.png", filename);
            let handle = asset_server.load(&path);
            section_sprites[idx].push(PickerUnit {
                section_name: section_order[idx].to_string(),
                col,
                row,
                handle,
                is_boat: false,
                visible: true,
                egui_texture: None,
                annotations_loaded: false,
            });
        }
    }

    for sprites in section_sprites {
        for sprite in sprites {
            picker.all.push((
                sprite.section_name.clone(),
                sprite.col,
                sprite.row,
                sprite.handle.clone(),
                sprite.is_boat,
            ));
            picker.available.push(sprite);
        }
    }
}

// ── Hex math helpers ───────────────────────────────────────────────────────────

fn hex_neighbors(coord: HexCoord) -> [HexCoord; 6] {
    let q = coord.q;
    let r = coord.r;
    [
        HexCoord::new(q + 1, r),
        HexCoord::new(q + 1, r + 1),
        HexCoord::new(q, r + 1),
        HexCoord::new(q - 1, r),
        HexCoord::new(q - 1, r - 1),
        HexCoord::new(q, r - 1),
    ]
}

// ── Left sidebar: list available units ─────────────────────────────────────────

fn load_egui_texture(
    ctx: &egui::Context,
    image: &Image,
    label: &str,
) -> Option<egui::TextureHandle> {
    let w = image.width() as usize;
    let h = image.height() as usize;
    if w == 0 || h == 0 {
        return None;
    }
    let data = image.data.as_ref()?;
    if data.len() < w * h * 4 {
        return None;
    }
    let pixels: Vec<egui::Color32> = data
        .chunks(4)
        .take(w * h)
        .map(|c| egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
        .collect();
    if pixels.len() != w * h {
        return None;
    }
    let color_image = egui::ColorImage {
        size: [w, h],
        pixels,
        source_size: egui::vec2(w as f32, h as f32),
    };
    Some(ctx.load_texture(label, color_image, egui::TextureOptions::LINEAR))
}

pub fn unit_picker_ui(
    mut contexts: EguiContexts,
    mode: Res<EditorMode>,
    mut picker: ResMut<UnitPicker>,
    mut state: ResMut<PickerState>,
    images: Res<Assets<Image>>,
    annotations: Option<Res<SpriteAnnotationsResource>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if *mode != EditorMode::Normal {
        return;
    }

    // — cache egui textures & look up is_boat from annotations —
    for unit in &mut picker.available {
        if unit.egui_texture.is_none()
            && let Some(image) = images.get(&unit.handle)
        {
            let label = format!("picker_{}_{}_{}", unit.section_name, unit.col, unit.row);
            unit.egui_texture = load_egui_texture(ctx, image, &label);
        }
        if !unit.annotations_loaded {
            if (!unit.is_boat || unit.visible)
                && let Some(ref ann) = annotations
            {
                let entry = ann
                    .0
                    .units
                    .get(&unit.section_name)
                    .and_then(|m| m.get(&(unit.col, unit.row)));
                if let Some(a) = entry {
                    if a.is_boat {
                        unit.is_boat = true;
                    }
                    if !a.is_unit {
                        unit.visible = false;
                    }
                }
            }
            unit.annotations_loaded = true;
        }
    }

    // — sidebar —
    egui::SidePanel::left("unit_picker_panel")
        .resizable(true)
        .default_width(200.0)
        .width_range(140.0..=320.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(8, 8)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::proportional(14.0));
            ui.label(
                egui::RichText::new("Unit Picker")
                    .size(16.0)
                    .color(egui::Color32::from_gray(220)),
            );
            ui.separator();
            ui.add_space(4.0);

            if picker.available.is_empty() {
                ui.colored_label(egui::Color32::from_gray(140), "all units placed");
            }

            let mut clicked_idx: Option<usize> = None;
            let mut drag_idx: Option<usize> = None;
            let sprite_size = 44.0;
            let margin = 2.0;
            let cell_size = sprite_size + margin * 2.0;

            // clear selection if the picked unit is now invisible
            if let PickerState::Placing { unit_idx, .. } = *state
                && picker.available.get(unit_idx).is_some_and(|u| !u.visible) {
                    *state = PickerState::Idle;
                }

            ui.style_mut().spacing.scroll.floating = false;
            egui::ScrollArea::vertical()
                .id_salt("unit_picker_scroll")
                .show(ui, |ui| {
                    let mut current_section = None::<&str>;

                    for idx in 0..picker.available.len() {
                        if !picker.available[idx].visible {
                            continue;
                        }
                        let section_name = picker.available[idx].section_name.as_str();

                        if Some(section_name) != current_section {
                            current_section = Some(section_name);
                            ui.add_space(6.0);
                            let display_name = section_name.replace('_', " ");
                            ui.label(
                                egui::RichText::new(display_name)
                                .size(13.0)
                                .color(egui::Color32::from_gray(180)),
                            );
                            ui.add_space(2.0);

                            ui.horizontal_wrapped(|ui| {
                                for j in idx..picker.available.len() {
                                    let next_section = &picker.available[j].section_name;
                                    if Some(next_section.as_str()) != current_section {
                                        break;
                                    }
                                    if !picker.available[j].visible {
                                        continue;
                                    }

                                    let is_selected = matches!(*state, PickerState::Placing { unit_idx, .. } if unit_idx == j);
                                    let unit = &picker.available[j];

                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::Vec2::new(cell_size, cell_size),
                                        egui::Sense::click_and_drag(),
                                    );

                                    let bg = if is_selected {
                                        egui::Color32::from_rgb(60, 100, 60)
                                    } else if response.hovered() {
                                        egui::Color32::from_rgb(60, 60, 80)
                                    } else {
                                        egui::Color32::from_gray(35)
                                    };
                                    let painter = ui.painter();
                                    painter.rect_filled(rect, 3.0, bg);

                                    if let Some(tex_id) = unit.egui_texture.as_ref().map(|t| t.id()) {
                                        let img_rect = egui::Rect::from_center_size(
                                            rect.center(),
                                            egui::Vec2::new(sprite_size, sprite_size),
                                        );
                                        painter.image(
                                            tex_id,
                                            img_rect,
                                            egui::Rect::from_min_max(
                                                egui::pos2(0.0, 0.0),
                                                egui::pos2(1.0, 1.0),
                                            ),
                                            egui::Color32::WHITE,
                                        );
                                    } else {
                                        painter.text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            format!("{}x{}", unit.col, unit.row),
                                            egui::FontId::proportional(10.0),
                                            egui::Color32::from_gray(120),
                                        );
                                    }

                                    if response.clicked() {
                                        clicked_idx = Some(j);
                                    }
                                    if response.drag_started() {
                                        drag_idx = Some(j);
                                    }
                                }
                            });
                        }
                    }
                });

            if let Some(idx) = clicked_idx {
                match *state {
                    PickerState::Placing { unit_idx, .. } if unit_idx == idx => {
                        *state = PickerState::Idle;
                    }
                    _ => {
                        *state = PickerState::Placing {
                            unit_idx: idx,
                            preview_hex: None,
                            preview_valid: false,
                            drag_drop: false,
                        };
                    }
                }
            }
            if let Some(idx) = drag_idx {
                *state = PickerState::Placing {
                    unit_idx: idx,
                    preview_hex: None,
                    preview_valid: false,
                    drag_drop: true,
                };
            }
        });

    // — ghost sprite at cursor when placing —
    if let PickerState::Placing { unit_idx, .. } = *state
        && let Some(unit) = picker.available.get(unit_idx)
        && let Some(tex_id) = unit.egui_texture.as_ref().map(|t| t.id())
        && let Some(pos) = ctx.pointer_latest_pos()
    {
        let ghost_size = 48.0;
        let ghost_rect = egui::Rect::from_center_size(pos, egui::Vec2::new(ghost_size, ghost_size));
        ctx.debug_painter().image(
            tex_id,
            ghost_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::from_white_alpha(180),
        );
    }
}

// ── Placement preview: green/red hex highlight ─────────────────────────────────

pub fn placement_preview_gizmo(
    mode: Res<EditorMode>,
    picker: Res<UnitPicker>,
    mut state: ResMut<PickerState>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    placed_units: Query<&PlacedUnit>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    mut gizmos: Gizmos,
) {
    if *mode != EditorMode::Normal {
        return;
    }
    let PickerState::Placing {
        unit_idx,
        ref mut preview_hex,
        ref mut preview_valid,
        ..
    } = *state
    else {
        return;
    };

    let Some(unit) = picker.available.get(unit_idx) else {
        *preview_hex = None;
        return;
    };

    let Some(hit) = raycast_ground(&windows, &cameras) else {
        *preview_hex = None;
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    // No preview outside the overlay-defined map.
    if !game_map.hexes.contains_key(&coord) {
        *preview_hex = None;
        return;
    }

    let occupied = placed_units.iter().any(|u| u.coord == coord);
    let valid = !occupied && coord_passable(&game_map, coord, unit.is_boat);
    *preview_hex = Some(coord);
    *preview_valid = valid;

    let pos = hex_world_pos(coord, origin, &overlay.params);
    let color = if valid {
        Color::srgb(0.0, 1.0, 0.0)
    } else {
        Color::srgb(1.0, 0.0, 0.0)
    };
    draw_hex_outline(&mut gizmos, pos, overlay.params.hex_size, color);
}

// ── Click handling: placement + movement ───────────────────────────────────────

pub fn handle_picker_clicks(
    mode: Res<EditorMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut picker: ResMut<UnitPicker>,
    mut state: ResMut<PickerState>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pending: ResMut<PendingEdits>,
) {
    if *mode != EditorMode::Normal {
        return;
    }
    let pressed = buttons.just_pressed(MouseButton::Left);
    let released = buttons.just_released(MouseButton::Left);
    if !pressed && !released {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if ctx.wants_pointer_input() {
        return;
    }

    let Some(hit) = raycast_ground(&windows, &cameras) else {
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    match std::mem::take(&mut *state) {
        PickerState::Idle => {
            if !pressed {
                *state = PickerState::Idle;
                return;
            }
            // click a placed unit to enter movement mode —
            // do not gate on game_map.hexes: a placed unit is its own proof
            if let Some((entity, _)) = placed_units.iter().find(|(_, u)| u.coord == coord) {
                *state = PickerState::Moving {
                    source: entity,
                    start_coord: coord,
                };
            } else {
                *state = PickerState::Idle;
            }
        }
        PickerState::Placing {
            unit_idx,
            drag_drop,
            ..
        } => {
            if released && !drag_drop {
                // click-release from picker — preserve placing state for next click
                *state = PickerState::Placing {
                    unit_idx,
                    preview_hex: None,
                    preview_valid: false,
                    drag_drop: false,
                };
                return;
            }

            let Some(unit) = picker.available.get(unit_idx) else {
                *state = PickerState::Idle;
                return;
            };

            let can_place = !placed_units.iter().any(|(_, u)| u.coord == coord)
                && coord_passable(&game_map, coord, unit.is_boat);

            if can_place {
                let pos = hex_world_pos(coord, origin, &overlay.params);
                let sprite_size = overlay.params.hex_size * 1.05;

                let unit = picker.available.remove(unit_idx);
                let material = materials.add(StandardMaterial {
                    base_color_texture: Some(unit.handle.clone()),
                    unlit: true,
                    alpha_mode: AlphaMode::Mask(0.1),
                    ..default()
                });

                commands.spawn((
                    PlacedUnit {
                        coord,
                        section_name: unit.section_name.clone(),
                        col: unit.col,
                        row: unit.row,
                        is_boat: unit.is_boat,
                    },
                    Mesh3d(meshes.add(Rectangle::new(sprite_size, sprite_size))),
                    MeshMaterial3d(material),
                    Transform::from_xyz(pos.x, 1.0, pos.z)
                        .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 2.0)),
                    Visibility::Visible,
                ));

                pending
                    .outgoing_broadcast
                    .push(NetMsg::Game(GameEvent::PlaceUnit {
                        section_name: unit.section_name.clone(),
                        col: unit.col,
                        row: unit.row,
                        coord_q: coord.q,
                        coord_r: coord.r,
                        is_boat: unit.is_boat,
                    }));
            }
            *state = PickerState::Idle;
        }
        PickerState::Moving {
            source,
            start_coord,
        } => {
            if released && coord == start_coord {
                // click-release on the same hex — stay in moving state
                *state = PickerState::Moving {
                    source,
                    start_coord,
                };
                return;
            }
            let Ok(placed) = placed_units.get(source) else {
                *state = PickerState::Idle;
                return;
            };
            let (_, placed) = placed;

            let target_occupied = placed_units.iter().any(|(_, u)| u.coord == coord);
            let passable = coord_passable(&game_map, coord, placed.is_boat);

            if hex_neighbors(placed.coord).contains(&coord) && !target_occupied && passable {
                let origin_pos = hex_world_pos(placed.coord, origin, &overlay.params);
                let target_pos = hex_world_pos(coord, origin, &overlay.params);

                commands.entity(source).insert(MovementAnimation {
                    from: Vec3::new(origin_pos.x, 1.0, origin_pos.z),
                    to: Vec3::new(target_pos.x, 1.0, target_pos.z),
                    progress: 0.0,
                    target_coord: coord,
                });

                pending
                    .outgoing_broadcast
                    .push(NetMsg::Game(GameEvent::MoveUnit {
                        section_name: placed.section_name.clone(),
                        col: placed.col,
                        row: placed.row,
                        to_q: coord.q,
                        to_r: coord.r,
                    }));
            }
            *state = PickerState::Idle;
        }
    }
}

// ── Movement overlay: light-green hex outlines ─────────────────────────────────

pub fn movement_overlay_gizmo(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    state: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    mut gizmos: Gizmos,
) {
    if *mode != EditorMode::Normal {
        return;
    }
    let PickerState::Moving { source, .. } = *state else {
        return;
    };
    let Ok((_, placed)) = placed_units.get(source) else {
        return;
    };

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);

    for neighbor in hex_neighbors(placed.coord) {
        if placed_units.iter().any(|(_, u)| u.coord == neighbor) {
            continue;
        }
        if !coord_passable(&game_map, neighbor, placed.is_boat) {
            continue;
        }
        let pos = hex_world_pos(neighbor, origin, &overlay.params);
        draw_hex_outline(
            &mut gizmos,
            pos,
            overlay.params.hex_size,
            Color::srgb(0.6, 1.0, 0.6),
        );
    }
}

// ── Animation: lerp unit movement ──────────────────────────────────────────────

pub fn animate_unit_movement(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut MovementAnimation,
        &mut PlacedUnit,
    )>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut anim, mut placed) in query.iter_mut() {
        anim.progress += time.delta_secs() / 0.3;
        if anim.progress >= 1.0 {
            transform.translation = anim.to;
            placed.coord = anim.target_coord;
            commands.entity(entity).remove::<MovementAnimation>();
        } else {
            let t = anim.progress;
            let ease = t * t * (3.0 - 2.0 * t);
            transform.translation = anim.from.lerp(anim.to, ease);
        }
    }
}

// ── Cancel placement/movement on right-click ──────────────────────────────────

pub fn cancel_placement(
    mode: Res<EditorMode>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut state: ResMut<PickerState>,
) {
    if *mode != EditorMode::Normal || !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_pointer_input()
    {
        return;
    }
    *state = PickerState::Idle;
}

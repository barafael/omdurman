use std::f32::consts::{FRAC_PI_6, PI};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hex::HexLayout;
use omdurman_map::{GameMap, clip_hexes_to_overlay, save_annotations_to_file};
use omdurman_types::{GridShape, OffsetVariant, Orientation, OverlayParams};

use crate::browser::SpriteAnnotationsResource;
use crate::editor::ANNOTATIONS_SAVE_PATH;
use crate::util::{adjusted_origin, hex_world_pos, hit_to_hex, raycast_ground};
use crate::{EditorMode, PendingEdits, RtsCamera};
use omdurman_net::NetMsg;

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

/// Adjustable hex grid overlay for layout calibration.
/// Active when the editor mode is `Overlay`.
#[derive(Resource, Default)]
pub struct HexOverlay {
    pub params: OverlayParams,
}

// ── Egui overlay panel ────────────────────────────────────────────────────────

pub fn overlay_ui(
    mut contexts: EguiContexts,
    mode: Res<EditorMode>,
    mut overlay: ResMut<HexOverlay>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    annotations: Option<Res<SpriteAnnotationsResource>>,
) {
    let ctx = guard_mode!(contexts, mode, Overlay);

    let mut params_changed = false;

    egui::SidePanel::right("overlay_panel")
        .resizable(true)
        .default_width(160.0)
        .width_range(120.0..=400.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_gray(45))
                .inner_margin(egui::Margin::symmetric(12, 12)),
        )
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            ui.horizontal(|ui| {
                ui.label("size");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.hex_size)
                            .speed(0.5)
                            .range(1.0..=200.0)
                            .clamp_existing_to_range(true),
                    )
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("x");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.offset_x)
                            .speed(1.0)
                            .clamp_existing_to_range(false),
                    )
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("y");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.offset_y)
                            .speed(1.0)
                            .clamp_existing_to_range(false),
                    )
                    .changed();
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("width");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.width)
                            .speed(1)
                            .range(1..=200)
                            .clamp_existing_to_range(true),
                    )
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("height");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.height)
                            .speed(1)
                            .range(1..=200)
                            .clamp_existing_to_range(true),
                    )
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("orientation");
                egui::ComboBox::from_id_salt("orientation")
                    .selected_text(format!("{:?}", overlay.params.orientation))
                    .show_ui(ui, |ui| {
                        params_changed |= ui
                            .selectable_value(
                                &mut overlay.params.orientation,
                                Orientation::Pointy,
                                "Pointy ⬢",
                            )
                            .changed();
                        params_changed |= ui
                            .selectable_value(
                                &mut overlay.params.orientation,
                                Orientation::Flat,
                                "Flat ⬣",
                            )
                            .changed();
                    });
            });
            ui.horizontal(|ui| {
                ui.label("offset");
                egui::ComboBox::from_id_salt("offset_variant")
                    .selected_text(format!("{:?}", overlay.params.offset_variant))
                    .show_ui(ui, |ui| match overlay.params.orientation {
                        Orientation::Pointy => {
                            params_changed |= ui
                                .selectable_value(
                                    &mut overlay.params.offset_variant,
                                    OffsetVariant::OddR,
                                    "OddR",
                                )
                                .changed();
                            params_changed |= ui
                                .selectable_value(
                                    &mut overlay.params.offset_variant,
                                    OffsetVariant::EvenR,
                                    "EvenR",
                                )
                                .changed();
                        }
                        Orientation::Flat => {
                            params_changed |= ui
                                .selectable_value(
                                    &mut overlay.params.offset_variant,
                                    OffsetVariant::OddQ,
                                    "OddQ",
                                )
                                .changed();
                            params_changed |= ui
                                .selectable_value(
                                    &mut overlay.params.offset_variant,
                                    OffsetVariant::EvenQ,
                                    "EvenQ",
                                )
                                .changed();
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("shape");
                egui::ComboBox::from_id_salt("shape")
                    .selected_text(format!("{:?}", overlay.params.shape))
                    .show_ui(ui, |ui| {
                        params_changed |= ui
                            .selectable_value(
                                &mut overlay.params.shape,
                                GridShape::Rectangle,
                                "Rectangle",
                            )
                            .changed();
                        params_changed |= ui
                            .selectable_value(
                                &mut overlay.params.shape,
                                GridShape::Parallelogram,
                                "Parallelogram",
                            )
                            .changed();
                    });
            });
            ui.label(format!("total: {} hexes", game_map.hexes.len()));
        });

    if params_changed {
        game_map.overlay = overlay.params.clone();
        clip_hexes_to_overlay(&mut game_map);
        pending
            .0
            .push(NetMsg::OverlayUpdate(overlay.params.clone()));
        if let Some(ref ann) = annotations {
            save_annotations_to_file(&game_map, &ann.0, ANNOTATIONS_SAVE_PATH);
        }
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
    mode: Res<EditorMode>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<SelectionMarker>>,
) {
    if matches!(*mode, EditorMode::Units | EditorMode::Secret) {
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
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    if game_map.hexes.contains_key(&coord) {
        let pos = hex_world_pos(coord, origin, &overlay.params);
        transform.translation = Vec3::new(pos.x, 0.5, pos.z);
        transform.scale = Vec3::splat(overlay.params.hex_size);
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}

// ── Hex grid outlines (overlay mode only) ────────────────────────────────────

pub fn draw_hex_debug(
    mode: Res<EditorMode>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mut gizmos: Gizmos,
) {
    if *mode != EditorMode::Overlay {
        return;
    }

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);

    for coord in game_map.hexes.keys() {
        let pos = hex_world_pos(*coord, origin, &overlay.params);
        draw_hex_outline(
            &mut gizmos,
            pos,
            overlay.params.hex_size,
            Color::srgb(1.0, 0.0, 0.0),
        );
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn hex_corners(center: Vec3, size: f32) -> [Vec3; 6] {
    std::array::from_fn(|k| {
        let angle = FRAC_PI_6 + k as f32 * PI / 3.0;
        Vec3::new(
            center.x + size * angle.cos(),
            center.y,
            center.z + size * angle.sin(),
        )
    })
}

pub fn draw_hex_outline(gizmos: &mut Gizmos, center: Vec3, size: f32, color: Color) {
    let verts = hex_corners(Vec3::new(center.x, 1.5, center.z), size);
    for i in 0..6 {
        gizmos.line(verts[i], verts[(i + 1) % 6], color);
    }
}

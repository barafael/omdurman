use std::f32::consts::{FRAC_PI_6, PI};

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::{GameMap, HexLayout, clip_hexes_to_overlay};
use omdurman_types::{GridShape, OffsetVariant, Orientation, OverlayParams};

use omdurman_hexmap::{adjusted_origin, hex_world_pos, hit_to_hex};

use crate::{EditorMode, HoveredHex, PendingEdits, camera::RtsCamera, util::raycast_ground};
use omdurman_net::{GameEvent, NetMsg};

// ── Map plane ─────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct MapPlane;

pub fn spawn_map_plane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Startup spawns the default Fall-of-Khartoum board; the plane is re-sized
    // and re-textured by `apply_map_data_to_plane` when a scenario selects a
    // board (§dual-map).
    let texture: Handle<Image> = asset_server.load("fall_of_khartoum_1885.png");
    commands.spawn((
        MapPlane,
        Name::new("MapPlane"),
        Mesh3d(meshes.add(Rectangle::new(
            omdurman_hexmap::IMG_W,
            omdurman_hexmap::IMG_H,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
    ));
}

/// Re-size and re-texture the existing map plane to a board's image and
/// dimensions (§dual-map). Used when a scenario selects a board or the editor
/// switches the active map.
pub fn apply_map_data_to_plane(
    plane: &Query<(&Mesh3d, &MeshMaterial3d<StandardMaterial>), With<MapPlane>>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    image: &str,
    img_w: f32,
    img_h: f32,
) {
    let Ok((mesh, material)) = plane.single() else {
        return;
    };
    if let Some(m) = meshes.get_mut(&mesh.0) {
        *m = Rectangle::new(img_w, img_h).into();
    }
    if let Some(mat) = materials.get_mut(&material.0) {
        mat.base_color_texture = Some(asset_server.load(image.to_string()));
    }
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
    mode: Res<State<EditorMode>>,
    mut overlay: ResMut<HexOverlay>,
    mut game_map: ResMut<GameMap>,
    mut pending: ResMut<PendingEdits>,
    mut dirty: ResMut<crate::AnnotationsDirty>,
    active: Res<crate::ActiveEditMap>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_overlay() {
        return;
    }

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
            ui.horizontal(|ui| {
                ui.label("rot°");
                // Fine grid rotation, ±4°, float-editable (drag, or click to type).
                params_changed |= ui
                    .add(
                        egui::Slider::new(&mut overlay.params.rotation_deg, -4.0..=4.0)
                            .step_by(0.0)
                            .fixed_decimals(2)
                            .clamping(egui::SliderClamping::Always),
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
                        params_changed |= ui
                            .selectable_value(
                                &mut overlay.params.shape,
                                GridShape::AlternatingRows,
                                "Alternating rows",
                            )
                            .changed();
                    });
            });
            // Parity toggle: only meaningful for the alternating-rows shape,
            // where it picks whether even or odd rows are the long ones.
            if overlay.params.shape == GridShape::AlternatingRows {
                ui.horizontal(|ui| {
                    ui.label("long rows");
                    let label = if overlay.params.long_rows_even {
                        "even (0,2,…)"
                    } else {
                        "odd (1,3,…)"
                    };
                    if ui.button(label).clicked() {
                        overlay.params.long_rows_even = !overlay.params.long_rows_even;
                        params_changed = true;
                    }
                });
            }
            ui.label(format!("total: {} hexes", game_map.hexes.len()));
        });

    if params_changed {
        game_map.overlay = overlay.params.clone();
        // Overlay defines the map shape: clip the in-memory map to match,
        // then persist the clipped map + overlay back to annotations.ron.
        clip_hexes_to_overlay(&mut game_map);
        pending
            .outgoing_broadcast
            .push(NetMsg::Game(GameEvent::OverlayUpdate(
                active.0,
                overlay.params.clone(),
            )));
        dirty.mark();
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

/// Moves a translucent hex marker to whichever map hex the cursor is over, and
/// records the hovered hex coordinate in [`HoveredHex`] for the UI.
pub fn update_selection_marker(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    mode: Res<State<EditorMode>>,
    mut marker: Query<(&mut Transform, &mut Visibility), With<SelectionMarker>>,
    mut hovered: ResMut<HoveredHex>,
) {
    // No hex hover marker in modes that don't act on whole hexes: the unit
    // sheet / event viewer (non-map scenes) and the Hexside editor, which shows
    // a per-segment hover instead (see `editor::draw_hexside_hover`).
    if mode.hides_hex_hover() {
        if let Ok((_, mut visibility)) = marker.single_mut() {
            *visibility = Visibility::Hidden;
        }
        hovered.0 = None;
        return;
    }
    let Ok((mut transform, mut visibility)) = marker.single_mut() else {
        hovered.0 = None;
        return;
    };
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        *visibility = Visibility::Hidden;
        hovered.0 = None;
        return;
    };
    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let coord = hit_to_hex(hit, origin, &overlay.params);

    if game_map.hexes.contains_key(&coord) {
        let pos = hex_world_pos(coord, origin, &overlay.params);
        transform.translation = Vec3::new(pos.x, 0.5, pos.z);
        transform.scale = Vec3::splat(overlay.params.hex_size);
        *visibility = Visibility::Visible;
        hovered.0 = Some(coord);
    } else {
        *visibility = Visibility::Hidden;
        hovered.0 = None;
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

// ── Hex ring mesh ────────────────────────────────────────────────────────

fn hex_ring_mesh() -> Mesh {
    let outer = 1.0;
    let inner = 0.92;
    let mut positions = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for i in 0..6 {
        let a0 = FRAC_PI_6 + i as f32 * PI / 3.0;
        let a1 = FRAC_PI_6 + (i + 1) as f32 * PI / 3.0;

        let o0 = Vec3::new(outer * a0.cos(), 0.0, outer * a0.sin());
        let o1 = Vec3::new(outer * a1.cos(), 0.0, outer * a1.sin());
        let i0 = Vec3::new(inner * a0.cos(), 0.0, inner * a0.sin());
        let i1 = Vec3::new(inner * a1.cos(), 0.0, inner * a1.sin());

        let base = positions.len() as u32;
        positions.extend([o0, o1, i0, i1]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    }

    let normals = vec![Vec3::Y; positions.len()];
    let uvs = vec![Vec2::ZERO; positions.len()];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

/// Shared mesh + colored materials for hex ring outlines.
#[derive(Resource)]
pub struct HexRingAssets {
    pub mesh: Handle<Mesh>,
    pub red: Handle<StandardMaterial>,
    pub green: Handle<StandardMaterial>,
    pub light_green: Handle<StandardMaterial>,
    pub orange: Handle<StandardMaterial>,
    pub cyan: Handle<StandardMaterial>,
}

pub fn spawn_hex_ring_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(hex_ring_mesh());
    let red = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    let green = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    let light_green = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 1.0, 0.6),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    let orange = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.55, 0.1),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    let cyan = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.9, 0.95),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    commands.insert_resource(HexRingAssets {
        mesh,
        red,
        green,
        light_green,
        orange,
        cyan,
    });
}

// ── Hex debug outlines (overlay mode) ───────────────────────────────────

#[derive(Component)]
pub(crate) struct HexDebugOutlines;

pub fn draw_hex_debug_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    existing: Query<Entity, With<HexDebugOutlines>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for e in &existing {
        commands.entity(e).despawn();
    }

    if game_map.hexes.is_empty() {
        return;
    }

    let origin = adjusted_origin(&layout, overlay.params.offset_x, overlay.params.offset_y);
    let size = overlay.params.hex_size;
    let outer = size;
    let inner = size * 0.92;
    let y = 1.5;

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    for coord in game_map.hexes.keys() {
        let pos = hex_world_pos(*coord, origin, &overlay.params);
        for i in 0..6 {
            let a0 = FRAC_PI_6 + i as f32 * PI / 3.0;
            let a1 = FRAC_PI_6 + (i + 1) as f32 * PI / 3.0;

            let o0 = Vec3::new(pos.x + outer * a0.cos(), y, pos.z + outer * a0.sin());
            let o1 = Vec3::new(pos.x + outer * a1.cos(), y, pos.z + outer * a1.sin());
            let i0 = Vec3::new(pos.x + inner * a0.cos(), y, pos.z + inner * a0.sin());
            let i1 = Vec3::new(pos.x + inner * a1.cos(), y, pos.z + inner * a1.sin());

            let base = positions.len() as u32;
            positions.extend([o0, o1, i0, i1]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
        }
    }

    let n = positions.len();
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::Y; n])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; n]);

    commands.spawn((
        HexDebugOutlines,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(assets.red.clone()),
        Visibility::Visible,
    ));
}

/// Registers all render-domain resources and systems: the map plane, hex
/// selection marker, overlay debug, and the overlay-control egui panel.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HexOverlay::default())
            .add_systems(
                Startup,
                (
                    spawn_map_plane,
                    spawn_selection_marker,
                    spawn_hex_ring_assets,
                ),
            )
            .add_systems(
                Update,
                (
                    draw_hex_debug_mesh.in_set(crate::OverlaySet),
                    update_selection_marker,
                ),
            )
            .add_systems(EguiPrimaryContextPass, (overlay_ui,));
    }
}

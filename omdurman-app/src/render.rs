use std::f32::consts::{FRAC_PI_6, PI};

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::{GameMap, HexLayout, clip_hexes_to_overlay};
use omdurman_types::{GridShape, OffsetVariant, Orientation, OverlayParams, Terrain};

use omdurman_hexmap::{hex_local_pos, hex_world_pos, hit_to_hex, local_to_world};

use crate::{
    AppMode, EditorTab, PendingEdits, camera::RtsCamera, editor::EditorToolState,
    util::raycast_ground,
};
use omdurman_net::{GameEvent, NetMsg};
use omdurman_types::HexCoord;

// -- Render resources -------------------------------------------------------

/// Written every frame by `render::update_selection_marker` with the hex
/// currently under the cursor (or `None` if no valid hex is hovered).
#[derive(Resource, Default)]
pub struct HoveredHex(pub Option<HexCoord>);

// -- Terrain overlay colour ----------------------------------------------------

/// Named palette colour for a terrain-type overlay. A typed enum (rather than
/// strum string props) so the terrain->colour mapping is total and checked.
/// Palette inspired by the Sudanese landscape (sand, Nile, khaki, earth).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TerrainColor {
    Sandy,
    DarkGreen,
    Blue,
    TanBrown,
    Brown,
    Tan,
    StoneGray,
    SwampGreen,
}

impl TerrainColor {
    fn rgba(self) -> [f32; 4] {
        match self {
            TerrainColor::Sandy => [0.90, 0.78, 0.40, 0.75],
            TerrainColor::DarkGreen => [0.28, 0.55, 0.15, 0.75],
            TerrainColor::Blue => [0.18, 0.55, 0.68, 0.75],
            TerrainColor::TanBrown => [0.72, 0.58, 0.38, 0.75],
            TerrainColor::Brown => [0.55, 0.40, 0.24, 0.75],
            TerrainColor::Tan => [0.82, 0.71, 0.52, 0.75],
            TerrainColor::StoneGray => [0.58, 0.58, 0.55, 0.75],
            TerrainColor::SwampGreen => [0.30, 0.42, 0.30, 0.75],
        }
    }
}

fn terrain_color(terrain: Terrain) -> TerrainColor {
    match terrain {
        Terrain::Clear { .. } => TerrainColor::Sandy,
        Terrain::Rough { .. } => TerrainColor::TanBrown,
        Terrain::Trees { .. } => TerrainColor::DarkGreen,
        Terrain::Swamp { .. } => TerrainColor::SwampGreen,
        Terrain::Nile { .. } => TerrainColor::Blue,
        Terrain::Hilltop { .. } => TerrainColor::Brown,
        Terrain::Huts { .. } => TerrainColor::Tan,
        Terrain::Building { .. } => TerrainColor::StoneGray,
    }
}

/// Return an RGBA colour suitable for a terrain-type overlay.
pub(crate) fn terrain_overlay_color(terrain: Terrain) -> [f32; 4] {
    terrain_color(terrain).rgba()
}

// -- Map plane -----------------------------------------------------------------

#[derive(Component)]
pub struct MapPlane;

/// Holds a handle per map-texture path so each board image is decoded and
/// uploaded once, kept resident across board switches, and re-used instantly
/// when switching back. Keyed by asset path; [`texture`](Self::texture) loads
/// on first request and returns the cached handle thereafter.
#[derive(Resource, Default)]
pub struct MapTextureCache(pub std::collections::HashMap<String, Handle<Image>>);

impl MapTextureCache {
    /// The handle for `image`, loading (and caching) it on first request.
    /// `AssetServer::load` already dedupes by path, so the win here is avoiding
    /// the repeated `load` call churn and giving us an explicit place to
    /// preload from.
    pub fn texture(&mut self, asset_server: &AssetServer, image: &str) -> Handle<Image> {
        self.0
            .entry(image.to_string())
            .or_insert_with(|| asset_server.load(image.to_string()))
            .clone()
    }
}

pub fn spawn_map_plane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    loaded: Res<crate::LoadedAnnotations>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Startup spawns the default Fall-of-Khartoum board; the plane is re-sized
    // and re-textured by `apply_map_data_to_plane` when a scenario selects a
    // board (§dual-map).
    //
    // Preload *both* boards' textures now so their (slow, ~30 MB) decode +
    // GPU upload overlaps the lobby/menu instead of stalling the first board
    // switch. The decode runs off the main thread; switching boards later then
    // just swaps to an already-resident handle (see `MapTextureCache`).
    let mut cache = MapTextureCache::default();
    for kind in [
        omdurman_types::MapKind::FallOfKhartoum,
        omdurman_types::MapKind::Campaign,
    ] {
        cache.texture(&asset_server, &loaded.0.map(kind).image);
    }
    let texture = cache.texture(&asset_server, "fall_of_khartoum_1885.webp");
    commands.insert_resource(cache);
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
    cache: &mut MapTextureCache,
    asset_server: &AssetServer,
    image: &str,
    img_w: f32,
    img_h: f32,
) {
    let Ok((mesh, material)) = plane.single() else {
        return;
    };
    if let Some(mut m) = meshes.get_mut(&mesh.0) {
        *m = Rectangle::new(img_w, img_h).into();
    }
    if let Some(mut mat) = materials.get_mut(&material.0) {
        // Re-use the already-decoded handle when switching back to a board.
        mat.base_color_texture = Some(cache.texture(asset_server, image));
    }
}

// -- Hex overlay resource ------------------------------------------------------

/// Adjustable hex grid overlay for layout calibration.
/// Active when the editor mode is `Overlay`.
#[derive(Resource, Default)]
pub struct HexOverlay {
    pub params: OverlayParams,
}

// -- Egui overlay panel --------------------------------------------------------

pub fn overlay_ui(
    mut contexts: EguiContexts,
    mode: EditorToolState,
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

    let mut __ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("overlay_panel"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    egui::Panel::right("overlay_panel")
        .resizable(true)
        .default_size(160.0)
        .size_range(120.0..=400.0)
        .frame(
            egui::Frame::default()
                .fill(crate::ui::panel_bg())
                .inner_margin(egui::Margin::symmetric(12, 12)),
        )
        .show(&mut __ui, |ui| {
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
                ui.label("rot deg");
                // Fine grid rotation, +/-4 deg, float-editable (drag, or click to type).
                // Hold Shift for a super-fine drag: swap the slider for a slow
                // DragValue so a whole drag sweep covers a fraction of a degree.
                let fine = ui.input(|i| i.modifiers.shift);
                let resp = if fine {
                    ui.add(
                        egui::DragValue::new(&mut overlay.params.rotation_deg)
                            .speed(0.002)
                            .range(-4.0..=4.0)
                            .fixed_decimals(3)
                            .clamp_existing_to_range(true),
                    )
                } else {
                    ui.add(
                        egui::Slider::new(&mut overlay.params.rotation_deg, -4.0..=4.0)
                            .step_by(0.0)
                            .fixed_decimals(2)
                            .clamping(egui::SliderClamping::Always),
                    )
                };
                params_changed |= resp.changed();
            });
            // Affine warp: anisotropic scale + shear, to register the lattice
            // against a scan that is stretched or photographed off-square.
            // Identity is aspect=1, shear=0. Hold Shift for super-fine drag.
            let fine = ui.input(|i| i.modifiers.shift);
            let (speed, decimals) = if fine { (0.0005, 4) } else { (0.002, 3) };
            ui.horizontal(|ui| {
                ui.label("aspect y");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.aspect_y)
                            .speed(speed)
                            .range(0.5..=2.0)
                            .fixed_decimals(decimals)
                            .clamp_existing_to_range(true),
                    )
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("shear x");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.shear_x)
                            .speed(speed)
                            .range(-0.3..=0.3)
                            .fixed_decimals(decimals)
                            .clamp_existing_to_range(true),
                    )
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("shear y");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.shear_y)
                            .speed(speed)
                            .range(-0.3..=0.3)
                            .fixed_decimals(decimals)
                            .clamp_existing_to_range(true),
                    )
                    .changed();
            });
            // Keystone: hex size grows/shrinks with distance from the origin
            // along x/y. Coefficients act over positions of hundreds of units, so
            // realistic values are tiny (well under 1e-3) -- the drag must be very
            // slow or a single pixel of drag jumps the whole grid. Hold Shift for
            // an even finer sweep.
            let (grad_speed, grad_decimals) = if fine { (0.000_001, 6) } else { (0.000_01, 5) };
            ui.horizontal(|ui| {
                ui.label("keystone x");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.size_grad_x)
                            .speed(grad_speed)
                            .range(-0.005..=0.005)
                            .fixed_decimals(grad_decimals)
                            .clamp_existing_to_range(true),
                    )
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("keystone y");
                params_changed |= ui
                    .add(
                        egui::DragValue::new(&mut overlay.params.size_grad_y)
                            .speed(grad_speed)
                            .range(-0.005..=0.005)
                            .fixed_decimals(grad_decimals)
                            .clamp_existing_to_range(true),
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
                                "Pointy [diamond]",
                            )
                            .changed();
                        params_changed |= ui
                            .selectable_value(
                                &mut overlay.params.orientation,
                                Orientation::Flat,
                                "Flat [hexagon]",
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
                        "even (0,2,...)"
                    } else {
                        "odd (1,3,...)"
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
            .push(NetMsg::Game(GameEvent::OverlayUpdate {
                map: active.0,
                params: overlay.params.clone(),
            }));
        dirty.mark();
    }
}

// -- Selection marker ----------------------------------------------------------

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
    mut marker: Query<(&mut Transform, &mut Visibility), With<SelectionMarker>>,
    mut hovered: ResMut<HoveredHex>,
) {
    let Ok((mut transform, mut visibility)) = marker.single_mut() else {
        hovered.0 = None;
        return;
    };
    let Some(hit) = raycast_ground(&windows, &cameras) else {
        *visibility = Visibility::Hidden;
        hovered.0 = None;
        return;
    };
    let origin = layout.adjusted_origin(&overlay.params);
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

pub fn hide_selection_marker(
    mut marker: Query<&mut Visibility, With<SelectionMarker>>,
    mut hovered: ResMut<HoveredHex>,
) {
    if let Ok(mut visibility) = marker.single_mut() {
        *visibility = Visibility::Hidden;
    }
    hovered.0 = None;
}

// -- Helpers -------------------------------------------------------------------

pub(crate) fn hex_corners(center: Vec3, size: f32) -> [Vec3; 6] {
    std::array::from_fn(|k| {
        let angle = FRAC_PI_6 + k as f32 * PI / 3.0;
        Vec3::new(
            center.x + size * angle.cos(),
            center.y,
            center.z + size * angle.sin(),
        )
    })
}

// -- Hex ring mesh --------------------------------------------------------

fn hex_ring_mesh() -> Mesh {
    let outer = 1.0;
    let inner = 0.96;
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
    pub brown: Handle<StandardMaterial>,
    pub gray: Handle<StandardMaterial>,
    pub yellow: Handle<StandardMaterial>,
    pub path_shadow: Handle<StandardMaterial>,
    pub fire_arrow: Handle<StandardMaterial>,
}

fn unlit_alpha_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    }
}

pub fn spawn_hex_ring_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(hex_ring_mesh());
    let red = materials.add(unlit_alpha_material(Color::srgb(1.0, 0.0, 0.0)));
    let green = materials.add(unlit_alpha_material(Color::srgb(0.0, 1.0, 0.0)));
    let light_green = materials.add(unlit_alpha_material(Color::srgb(0.6, 1.0, 0.6)));
    let orange = materials.add(unlit_alpha_material(Color::srgb(1.0, 0.55, 0.1)));
    let brown = materials.add(unlit_alpha_material(Color::srgb(0.35, 0.22, 0.1)));
    let gray = materials.add(unlit_alpha_material(Color::srgb(0.4, 0.4, 0.4)));
    let yellow = materials.add(unlit_alpha_material(Color::srgba(1.0, 0.85, 0.0, 0.4)));
    let path_shadow =
        materials.add(unlit_alpha_material(Color::srgba(0.45, 0.55, 0.95, 0.18)));
    let fire_arrow =
        materials.add(unlit_alpha_material(Color::srgba(0.9, 0.15, 0.1, 0.45)));
    commands.insert_resource(HexRingAssets {
        mesh,
        red,
        green,
        light_green,
        orange,
        brown,
        gray,
        yellow,
        path_shadow,
        fire_arrow,
    });
}

// -- Movement-path arrow mesh ---------------------------------------------

/// A unit-length arrow lying in the XZ plane, tail at the origin and tip at
/// `(0, 0, 1)`. Built once and reused for every path segment: the spawn system
/// scales its length and rotates `+Z` onto the segment's heading (via
/// `Quat::from_rotation_arc`, matching the Nile-arrow convention), so one mesh
/// serves all arrows regardless of length or direction.
///
/// Geometry: a thin rectangular shaft from `z=0` to `z=SHAFT_END`, then a
/// triangular head from `SHAFT_END` to the tip at `z=1`. Width is along X.
fn arrow_mesh() -> Mesh {
    const SHAFT_END: f32 = 0.68;
    const SHAFT_HALF: f32 = 0.09;
    const HEAD_HALF: f32 = 0.24;

    let positions: Vec<Vec3> = vec![
        // shaft quad
        Vec3::new(-SHAFT_HALF, 0.0, 0.0),
        Vec3::new(SHAFT_HALF, 0.0, 0.0),
        Vec3::new(SHAFT_HALF, 0.0, SHAFT_END),
        Vec3::new(-SHAFT_HALF, 0.0, SHAFT_END),
        // head triangle
        Vec3::new(-HEAD_HALF, 0.0, SHAFT_END),
        Vec3::new(HEAD_HALF, 0.0, SHAFT_END),
        Vec3::new(0.0, 0.0, 1.0),
    ];
    // Wind both faces CCW when viewed from +Y (top); cull_mode is None anyway.
    let indices = vec![0u32, 2, 1, 0, 3, 2, 4, 6, 5];
    let n = positions.len();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::Y; n])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; n])
}

/// Shared arrow mesh + dim/highlight materials for movement-path rendering.
#[derive(Resource)]
pub struct MovementArrowAssets {
    pub mesh: Handle<Mesh>,
    /// Faint fill for paths not currently hovered.
    pub dim: Handle<StandardMaterial>,
    /// Bright fill for the path under the cursor.
    pub bright: Handle<StandardMaterial>,
}

pub fn spawn_movement_arrow_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(arrow_mesh());
    // Mild orange: dim (faint) for idle paths, brighter for the hovered one.
    let dim = materials.add(unlit_alpha_material(Color::srgba(0.85, 0.5, 0.2, 0.45)));
    let bright = materials.add(unlit_alpha_material(Color::srgba(0.95, 0.55, 0.2, 0.95)));
    commands.insert_resource(MovementArrowAssets { mesh, dim, bright });
}

// -- Hex debug outlines (overlay mode) -----------------------------------

#[derive(Component)]
pub(crate) struct HexDebugOutlines;

fn hide_hex_debug_outlines(
    mut commands: Commands,
    existing: Query<Entity, With<HexDebugOutlines>>,
) {
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);
}

pub fn draw_hex_debug_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    existing: Query<Entity, With<HexDebugOutlines>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let existing: Vec<Entity> = existing.iter().collect();
    crate::ui::despawn_all(&mut commands, &existing);

    if game_map.hexes.is_empty() {
        return;
    }

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    let outer = size;
    let inner = size * 0.96;
    let y = 1.5;

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    // Build each ring corner in *local* lattice space (centre + corner offset)
    // and push it through the full warp, so the outlines shear and grow with the
    // affine/keystone params instead of staying regular hexagons on warped
    // centres.
    let corner = |c: Vec3, radius: f32, angle: f32| {
        let p = local_to_world(
            c.x + radius * angle.cos(),
            c.z + radius * angle.sin(),
            origin,
            &overlay.params,
        );
        Vec3::new(p.x, y, p.z)
    };
    for coord in game_map.hexes.keys() {
        let c = hex_local_pos(*coord, &overlay.params);
        for i in 0..6 {
            let a0 = FRAC_PI_6 + i as f32 * PI / 3.0;
            let a1 = FRAC_PI_6 + (i + 1) as f32 * PI / 3.0;

            let o0 = corner(c, outer, a0);
            let o1 = corner(c, outer, a1);
            let i0 = corner(c, inner, a0);
            let i1 = corner(c, inner, a1);

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
            .insert_resource(HoveredHex::default())
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
                    update_selection_marker.run_if(crate::hex_hover_visible),
                ),
            )
            // The hex-hover selection marker is hidden on entering the editor
            // tabs that suppress it (hexside/unit-sheet/event-viewer); the
            // per-frame `update_selection_marker` is itself gated by
            // `hex_hover_visible`, this just clears a stale marker on the switch.
            .add_systems(OnEnter(EditorTab::Hexside), hide_selection_marker)
            .add_systems(OnEnter(EditorTab::UnitSheet), hide_selection_marker)
            .add_systems(OnEnter(EditorTab::EventViewer), hide_selection_marker)
            .add_systems(OnEnter(EditorTab::Charts), hide_selection_marker)
            // The overlay-calibration debug outlines only belong to the Overlay
            // tab; clear them when that tab or the editor is left.
            .add_systems(OnExit(EditorTab::Overlay), hide_hex_debug_outlines)
            .add_systems(OnExit(AppMode::Editor), hide_hex_debug_outlines)
            .add_systems(EguiPrimaryContextPass, (overlay_ui,));
    }
}

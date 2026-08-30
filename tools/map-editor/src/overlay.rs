//! The Overlay tab: hex-grid calibration panel + warp-aware grid outlines.

use std::f32::consts::{FRAC_PI_6, PI};

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use omdurman_hexmap::{GameMap, HexLayout, HexOverlay, hex_local_pos, local_to_world};
use omdurman_types::{GridShape, OffsetVariant, Orientation};

use crate::{
    board::{ActiveEditMap, LoadedAnnotations, RingAssets},
    editor::EditorToolState,
    edits::{self, EditCtx},
    state::EditorTab,
    ui_plugin::PanelUiSet,
};

pub fn overlay_ui(
    mut contexts: EguiContexts,
    mode: EditorToolState,
    mut loaded: ResMut<LoadedAnnotations>,
    mut overlay: ResMut<HexOverlay>,
    mut game_map: ResMut<GameMap>,
    active: Res<ActiveEditMap>,
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
    let __panel = egui::Panel::right("overlay_panel")
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
    let _ = ctx;

    if params_changed {
        let params = overlay.params.clone();
        edits::apply_overlay_update(
            &mut EditCtx {
                loaded: &mut loaded,
                game_map: &mut game_map,
                overlay: &mut overlay,
                active: &active,
            },
            &params,
        );
    }
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
    assets: Res<RingAssets>,
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

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            draw_hex_debug_mesh.run_if(in_state(EditorTab::Overlay)),
        )
        .add_systems(OnExit(EditorTab::Overlay), hide_hex_debug_outlines)
        .add_systems(EguiPrimaryContextPass, overlay_ui.in_set(PanelUiSet));
    }
}

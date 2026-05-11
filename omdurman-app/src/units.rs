use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use serde::{Deserialize, Serialize};

use crate::RtsCamera;

const UNITS_IMG_W: f32 = 1233.0;
const UNITS_IMG_H: f32 = 1593.0;

fn pixel_to_world(px: f32, py: f32) -> Vec3 {
    Vec3::new(px - UNITS_IMG_W * 0.5, 0.0, py - UNITS_IMG_H * 0.5)
}

const UNIT_GRIDS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/unit_grids.ron");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UnitGrid {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub cols: u32,
    pub rows: u32,
}

#[derive(Resource, Debug)]
pub struct UnitViewer {
    pub visible: bool,
    pub grids: Vec<UnitGrid>,
}

#[derive(Component)]
pub struct UnitsPlane;

impl UnitViewer {
    pub fn load_or_default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        if let Ok(contents) = std::fs::read_to_string(UNIT_GRIDS_PATH) {
            if let Ok(grids) = ron::from_str::<Vec<UnitGrid>>(&contents) {
                bevy::log::info!("loaded {} unit grids", grids.len());
                return Self { visible: false, grids };
            }
        }
        Self {
            visible: false,
            grids: default_grids(),
        }
    }
}

fn default_grids() -> Vec<UnitGrid> {
    let uw = 88.0;
    let uh = 76.0;
    vec![
        UnitGrid { name: "Talasha".into(), x: 60.0, y: 60.0, width: 7.0 * uw, height: 2.0 * uh, cols: 7, rows: 2 },
        UnitGrid { name: "Khalifa Abdullay".into(), x: 700.0, y: 60.0, width: 3.0 * uw, height: 2.0 * uh, cols: 3, rows: 2 },
        UnitGrid { name: "Sherif".into(), x: 60.0, y: 260.0, width: 3.0 * uw, height: 2.0 * uh, cols: 3, rows: 2 },
        UnitGrid { name: "upper green".into(), x: 700.0, y: 260.0, width: 8.0 * uw, height: 2.0 * uh, cols: 8, rows: 2 },
        UnitGrid { name: "lower green".into(), x: 60.0, y: 460.0, width: 8.0 * uw, height: 2.0 * uh, cols: 8, rows: 2 },
        UnitGrid { name: "upper Jaalin".into(), x: 700.0, y: 460.0, width: 7.0 * uw, height: 2.0 * uh, cols: 7, rows: 2 },
        UnitGrid { name: "lower Jaalin".into(), x: 60.0, y: 660.0, width: 7.0 * uw, height: 2.0 * uh, cols: 7, rows: 2 },
        UnitGrid { name: "Hadendowa".into(), x: 700.0, y: 660.0, width: 8.0 * uw, height: 2.0 * uh, cols: 8, rows: 2 },
        UnitGrid { name: "Hadendowa Guns".into(), x: 60.0, y: 860.0, width: 8.0 * uw, height: 2.0 * uh, cols: 8, rows: 2 },
        UnitGrid { name: "Baggara".into(), x: 700.0, y: 860.0, width: 7.0 * uw, height: 2.0 * uh, cols: 7, rows: 2 },
        UnitGrid { name: "British Boats".into(), x: 60.0, y: 1060.0, width: 8.0 * uw, height: 2.0 * uh, cols: 8, rows: 2 },
        UnitGrid { name: "Ali Wad Helu".into(), x: 700.0, y: 1060.0, width: 7.0 * uw, height: 2.0 * uh, cols: 7, rows: 2 },
        UnitGrid { name: "British Army".into(), x: 60.0, y: 1260.0, width: 8.0 * uw, height: 2.0 * uh, cols: 8, rows: 2 },
        UnitGrid { name: "Kitchener".into(), x: 700.0, y: 1260.0, width: 8.0 * uw, height: 2.0 * uh, cols: 8, rows: 2 },
        UnitGrid { name: "Sheik El Din".into(), x: 60.0, y: 1460.0, width: 7.0 * uw, height: 2.0 * uh, cols: 7, rows: 2 },
        UnitGrid { name: "Jehadia".into(), x: 700.0, y: 1460.0, width: 7.0 * uw, height: 2.0 * uh, cols: 7, rows: 2 },
        UnitGrid { name: "Egyptian Army".into(), x: 60.0, y: 1660.0, width: 8.0 * uw, height: 2.0 * uh, cols: 8, rows: 2 },
    ]
}

pub fn spawn_units_plane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let texture: Handle<Image> = asset_server.load("units.png");
    commands.spawn((
        UnitsPlane,
        Name::new("UnitsPlane"),
        Mesh3d(meshes.add(Rectangle::new(UNITS_IMG_W, UNITS_IMG_H))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture),
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
        Visibility::Hidden,
    ));
}

pub fn units_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
    mut viewer: ResMut<UnitViewer>,
    mut vis_set: ParamSet<(
        Query<&mut Visibility, With<UnitsPlane>>,
        Query<&mut Visibility, With<crate::render::MapPlane>>,
    )>,
) {
    if let Ok(ctx) = contexts.ctx_mut()
        && ctx.wants_keyboard_input()
    {
        return;
    }
    if keys.just_pressed(KeyCode::Digit3)
        && keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
        viewer.visible = !viewer.visible;
        if let Ok(mut vis) = vis_set.p0().single_mut() {
            *vis = if viewer.visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        if let Ok(mut vis) = vis_set.p1().single_mut() {
            *vis = if viewer.visible {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }
}

pub fn draw_unit_grids(viewer: Res<UnitViewer>, mut gizmos: Gizmos) {
    if !viewer.visible {
        return;
    }
    for grid in &viewer.grids {
        let tl = pixel_to_world(grid.x, grid.y);
        let br = pixel_to_world(grid.x + grid.width, grid.y + grid.height);
        let color = Color::srgb(1.0, 0.0, 0.0);
        let sub = Color::srgb(0.6, 0.0, 0.0);
        let y = 1.0;

        let left = tl.x;
        let right = br.x;
        let top = tl.z;
        let bottom = br.z;

        // outer border
        gizmos.line(Vec3::new(left, y, top), Vec3::new(right, y, top), color);
        gizmos.line(Vec3::new(right, y, top), Vec3::new(right, y, bottom), color);
        gizmos.line(Vec3::new(right, y, bottom), Vec3::new(left, y, bottom), color);
        gizmos.line(Vec3::new(left, y, bottom), Vec3::new(left, y, top), color);

        if grid.cols > 1 {
            let col_w = grid.width / grid.cols as f32;
            for c in 1..grid.cols {
                let cx = pixel_to_world(grid.x + c as f32 * col_w, grid.y).x;
                gizmos.line(Vec3::new(cx, y, top), Vec3::new(cx, y, bottom), sub);
            }
        }
        if grid.rows > 1 {
            let row_h = grid.height / grid.rows as f32;
            for r in 1..grid.rows {
                let cz = pixel_to_world(grid.x, grid.y + r as f32 * row_h).z;
                gizmos.line(Vec3::new(left, y, cz), Vec3::new(right, y, cz), sub);
            }
        }
    }
}

pub fn unit_grids_ui(
    mut contexts: EguiContexts,
    mut viewer: ResMut<UnitViewer>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !viewer.visible {
        return;
    }
    let mut changed = false;

    egui::Window::new("unit grids")
        .default_pos([14.0, 14.0])
        .resizable(false)
        .title_bar(true)
        .show(ctx, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            egui::ScrollArea::vertical().max_height(600.0).show(ui, |ui| {
                let mut remove_idx = None;
                for (i, grid) in viewer.grids.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(&grid.name);
                            if ui.button("x").clicked() {
                                remove_idx = Some(i);
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("x");
                            changed |= ui.add(egui::DragValue::new(&mut grid.x).speed(1.0)).changed();
                            ui.label("y");
                            changed |= ui.add(egui::DragValue::new(&mut grid.y).speed(1.0)).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("w");
                            changed |= ui.add(
                                egui::DragValue::new(&mut grid.width)
                                    .speed(1.0)
                                    .range(1.0..=2000.0)
                                    .clamp_existing_to_range(false),
                            ).changed();
                            ui.label("h");
                            changed |= ui.add(
                                egui::DragValue::new(&mut grid.height)
                                    .speed(1.0)
                                    .range(1.0..=2000.0)
                                    .clamp_existing_to_range(false),
                            ).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("cols");
                            changed |= ui.add(egui::DragValue::new(&mut grid.cols).speed(1).range(1..=50)).changed();
                            ui.label("rows");
                            changed |= ui.add(egui::DragValue::new(&mut grid.rows).speed(1).range(1..=50)).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("name");
                            changed |= ui.add(egui::TextEdit::singleline(&mut grid.name).desired_width(120.0)).changed();
                        });
                    });
                    ui.add_space(2.0);
                }
                if let Some(idx) = remove_idx {
                    viewer.grids.remove(idx);
                    changed = true;
                }
            });
        });

    if changed {
        save_unit_grids(&viewer.grids);
    }
}

pub fn unit_grid_labels(
    mut contexts: EguiContexts,
    viewer: Res<UnitViewer>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
) {
    if !viewer.visible {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(vp_size) = camera.logical_viewport_size() else {
        return;
    };

    for (i, grid) in viewer.grids.iter().enumerate() {
        let world_pos = pixel_to_world(grid.x + grid.width * 0.5, grid.y);
        let Ok(screen) = camera.world_to_viewport(cam_transform, world_pos) else {
            continue;
        };
        if screen.x < 0.0 || screen.x > vp_size.x || screen.y < 0.0 || screen.y > vp_size.y {
            continue;
        }
        egui::Area::new(egui::Id::new(("ug", i)))
            .fixed_pos(egui::pos2(screen.x - 40.0, screen.y - 16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
                ui.colored_label(egui::Color32::WHITE, &grid.name);
            });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_unit_grids(grids: &[UnitGrid]) {
    match ron::to_string(grids) {
        Ok(contents) => match std::fs::write(UNIT_GRIDS_PATH, contents) {
            Ok(()) => bevy::log::info!("saved {} unit grids to {UNIT_GRIDS_PATH}", grids.len()),
            Err(e) => bevy::log::error!("failed to write {UNIT_GRIDS_PATH}: {e}"),
        },
        Err(e) => bevy::log::error!("failed to serialize unit grids: {e}"),
    }
}

#[cfg(target_arch = "wasm32")]
fn save_unit_grids(_grids: &[UnitGrid]) {}

/// A single counter cell cut from a grid.
#[derive(Debug, Clone, PartialEq)]
pub struct CounterCell {
    pub unit: String,
    pub col: u32,
    pub row: u32,
    /// Pixel rect on the units image (x, y, width, height).
    pub rect: (f32, f32, f32, f32),
}

/// Cuts every grid into its individual counter cells.
pub fn cut_grids(grids: &[UnitGrid]) -> Vec<CounterCell> {
    let mut cells = Vec::new();
    for g in grids {
        let cw = g.width / g.cols as f32;
        let ch = g.height / g.rows as f32;
        for row in 0..g.rows {
            for col in 0..g.cols {
                cells.push(CounterCell {
                    unit: g.name.clone(),
                    col,
                    row,
                    rect: (
                        g.x + col as f32 * cw,
                        g.y + row as f32 * ch,
                        cw,
                        ch,
                    ),
                });
            }
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_grids_from_file() {
        let contents = std::fs::read_to_string(UNIT_GRIDS_PATH)
            .expect("unit_grids.ron should exist at compile-time path");
        let grids: Vec<UnitGrid> = ron::from_str(&contents)
            .expect("unit_grids.ron should be valid ron");

        assert!(!grids.is_empty(), "at least one grid");

        let cells = cut_grids(&grids);

        // every cell is within the image
        for cell in &cells {
            assert!(cell.rect.0 >= 0.0, "{} col {} x", cell.unit, cell.col);
            assert!(cell.rect.1 >= 0.0, "{} row {} y", cell.unit, cell.row);
            assert!(cell.rect.2 > 0.0, "{} col {} width", cell.unit, cell.col);
            assert!(cell.rect.3 > 0.0, "{} row {} height", cell.unit, cell.row);
            assert!(
                cell.rect.0 + cell.rect.2 <= UNITS_IMG_W + 1.0,
                "{} col {} right edge {} exceeds image width {}",
                cell.unit, cell.col, cell.rect.0 + cell.rect.2, UNITS_IMG_W,
            );
            assert!(
                cell.rect.1 + cell.rect.3 <= UNITS_IMG_H + 1.0,
                "{} row {} bottom edge {} exceeds image height {}",
                cell.unit, cell.row, cell.rect.1 + cell.rect.3, UNITS_IMG_H,
            );
        }

        // total cell count
        let expected: usize = grids.iter().map(|g| (g.cols * g.rows) as usize).sum();
        assert_eq!(cells.len(), expected);

        // cells tile to fill each grid exactly
        for g in &grids {
            // first cell starts at grid origin
            let first = cells.iter().find(|c| c.unit == g.name && c.col == 0 && c.row == 0);
            assert!(first.is_some(), "{} missing cell 0,0", g.name);
            let first = first.unwrap();
            assert!((first.rect.0 - g.x).abs() < 0.001, "{} origin x", g.name);
            assert!((first.rect.1 - g.y).abs() < 0.001, "{} origin y", g.name);
            // last cell ends at grid edge
            let last = cells.iter().find(|c| c.unit == g.name && c.col == g.cols - 1 && c.row == g.rows - 1);
            assert!(last.is_some(), "{} missing cell {},{}", g.name, g.cols - 1, g.rows - 1);
            let last = last.unwrap();
            assert!((last.rect.0 + last.rect.2 - (g.x + g.width)).abs() < 0.001, "{} right edge", g.name);
            assert!((last.rect.1 + last.rect.3 - (g.y + g.height)).abs() < 0.001, "{} bottom edge", g.name);
        }

        // print summary
        println!("── {} grids, {} counters ──", grids.len(), cells.len());
        for g in &grids {
            let count: usize = cells.iter().filter(|c| c.unit == g.name).count();
            println!("  {:<20}  {}×{} = {} cells  @ ({:.0},{:.0}) {}×{}",
                g.name, g.cols, g.rows, count, g.x, g.y, g.width, g.height);
        }
    }
}

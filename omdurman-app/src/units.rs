use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    PendingEdits, SidebarClip, browser::SpriteBrowser, camera::RtsCamera, editor::EditorToolState,
};
use omdurman_net::{GameEvent, NetMsg};
use omdurman_types::SectionName;

const UNITS_IMG_W: f32 = 2967.0;
const UNITS_IMG_H: f32 = 3893.0;

fn pixel_to_world(px: f32, py: f32) -> Vec3 {
    Vec3::new(px - UNITS_IMG_W * 0.5, 0.0, py - UNITS_IMG_H * 0.5)
}

use omdurman_types::UnitGrid;

/// The raw units sheet. It lives in `editor-assets/` (outside `assets/`, so
/// Trunk's `copy-dir` never ships it to the web build) and is used only by the
/// native editor: as the backdrop behind the grid rectangles and as the source
/// the sprites are cut from. Stored as high-quality WebP (q92) — the `image`
/// crate decodes it fine, and at sprite-display size the difference from the
/// original scan is imperceptible.
#[cfg(not(target_arch = "wasm32"))]
const UNITS_SHEET_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/editor-assets/units.webp");

#[derive(Resource, Debug)]
pub struct UnitViewer {
    pub grids: Vec<UnitGrid>,
    /// Tracks whether grid rectangles have been edited since the last
    /// remote/persisted update. Used to batch network updates to drag-end.
    pub grids_dirty: bool,
    /// Indices of grids edited since the last flush, so drag-end only re-cuts
    /// the sprites that actually changed (re-cutting all ~238 is slow).
    pub dirty_grids: std::collections::HashSet<usize>,
}

#[derive(Component)]
pub struct UnitsPlane;

impl UnitViewer {
    pub fn load_or_default() -> Self {
        let contents = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/unit_grids.ron"
        ));
        match ron::from_str::<Vec<UnitGrid>>(contents) {
            Ok(grids) => {
                bevy::log::info!("loaded {} unit grids", grids.len());
                Self {
                    grids,
                    grids_dirty: false,
                    dirty_grids: std::collections::HashSet::new(),
                }
            }
            Err(e) => {
                bevy::log::error!("failed to parse embedded unit_grids.ron: {e}");
                Self {
                    grids: vec![],
                    grids_dirty: false,
                    dirty_grids: std::collections::HashSet::new(),
                }
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
pub fn spawn_units_plane(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // The units sheet is a native-editor-only backdrop (drawn behind the grid
    // rectangles in UnitSheet mode). The web build has no editor and never
    // unhides this plane, so it gets no texture there — that keeps the ~21 MB
    // sheet out of the web build entirely (it isn't even under `assets/`).
    //
    // On native we load it directly with the `image` crate rather than the
    // asset server, because the asset server is rooted at `assets/` and can't
    // reach `editor-assets/`.
    #[cfg(not(target_arch = "wasm32"))]
    let base_color_texture = load_units_sheet_image(&mut images);
    #[cfg(target_arch = "wasm32")]
    let base_color_texture = None;
    commands.spawn((
        UnitsPlane,
        Name::new("UnitsPlane"),
        Mesh3d(meshes.add(Rectangle::new(UNITS_IMG_W, UNITS_IMG_H))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture,
            unlit: true,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0)),
        Visibility::Hidden,
    ));
}

/// Decode the units sheet from `editor-assets/` and register it as a Bevy
/// image. Returns `None` (logging the error) if the file is missing or fails to
/// decode, so a broken editor backdrop never takes down startup.
#[cfg(not(target_arch = "wasm32"))]
fn load_units_sheet_image(images: &mut Assets<Image>) -> Option<Handle<Image>> {
    use bevy::asset::RenderAssetUsages;

    let dyn_img = match image::open(UNITS_SHEET_PATH) {
        Ok(img) => img,
        Err(e) => {
            bevy::log::error!("failed to open {UNITS_SHEET_PATH} for the units backdrop: {e}");
            return None;
        }
    };
    let image = Image::from_dynamic(dyn_img, true, RenderAssetUsages::RENDER_WORLD);
    Some(images.add(image))
}

pub fn draw_unit_grids(
    mode: EditorToolState,
    viewer: Res<UnitViewer>,
    browser: Res<SpriteBrowser>,
    mut gizmos: Gizmos,
) {
    if !mode.is_unit_sheet() {
        return;
    }

    // If a sprite/section is selected *while in UnitSheet mode*, only
    // highlight the matching grid. When entering UnitSheet the second time we
    // want all rectangles visible again even if Units mode left a selection
    // behind, so we ignore selections made in other modes.
    // Grid names use spaces (e.g. "upper green"), sections use underscores
    // (e.g. "upper_green"). Compare by the canonical SectionName, not the
    // human display name -- several sections now share a display name (the
    // "green"/"Jaalin" sheet sections are simply Mulazmin/Jaalin), so matching
    // on display text would wrongly highlight every same-named grid.
    let selected_name = if mode.is_unit_sheet() {
        browser.selected_sprite.as_ref().map(|s| s.section_name)
    } else {
        None
    };

    for grid in &viewer.grids {
        if let Some(section_name) = selected_name
            && grid.name.replace(' ', "_").parse::<SectionName>() != Ok(section_name)
        {
            continue;
        }
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
        gizmos.line(
            Vec3::new(right, y, bottom),
            Vec3::new(left, y, bottom),
            color,
        );
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
    mode: EditorToolState,
    mut viewer: ResMut<UnitViewer>,
    mut clip: ResMut<SidebarClip>,
    mut pending: ResMut<PendingEdits>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_unit_sheet() {
        clip.right_sidebar = None;
        return;
    }

    let mut changed = false;
    // Indices of grids touched this frame (to re-cut only those at drag-end).
    let mut edited_grids: Vec<usize> = Vec::new();

    let mut __ui = egui::Ui::new(
        ctx.clone(),
        egui::Id::new("units_panel"),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    let response = egui::Panel::right("unit_grids_panel")
        .resizable(true)
        .default_size(300.0)
        .size_range(200.0..=600.0)
        .frame(
            egui::Frame::default()
                .fill(crate::ui::panel_bg())
                .inner_margin(egui::Margin::symmetric(12, 12)),
        )
        .show(&mut __ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(13.0));
            if ui.button("Clear + Re-Crop All").clicked() {
                clear_sprites_dir();
                cut_sprites_for_grids(&viewer.grids);
            }
            ui.add_space(4.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, grid) in viewer.grids.iter_mut().enumerate() {
                    let mut grid_changed = false;
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(&grid.name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("x");
                            grid_changed |= ui
                                .add(egui::DragValue::new(&mut grid.x).speed(1.0))
                                .changed();
                            ui.label("y");
                            grid_changed |= ui
                                .add(egui::DragValue::new(&mut grid.y).speed(1.0))
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("w");
                            grid_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut grid.width)
                                        .speed(1.0)
                                        .range(1.0..=2000.0)
                                        .clamp_existing_to_range(false),
                                )
                                .changed();
                            ui.label("h");
                            grid_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut grid.height)
                                        .speed(1.0)
                                        .range(1.0..=2000.0)
                                        .clamp_existing_to_range(false),
                                )
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("name");
                            grid_changed |= ui
                                .add(
                                    egui::TextEdit::singleline(&mut grid.name)
                                        .desired_width(f32::INFINITY),
                                )
                                .changed();
                        });
                    });
                    if grid_changed {
                        changed = true;
                        edited_grids.push(idx);
                    }
                    ui.add_space(2.0);
                }
            });
        });
    clip.right_sidebar = Some(response.response.rect);
    crate::ui_plugin::register_panel_rect(ctx, response.response.rect);

    // Always apply grid edits locally so rectangles on the unit sheet update
    // in real time while dragging. Remember which grids changed so drag-end
    // only re-cuts those sprites, not all ~238.
    if changed {
        viewer.grids_dirty = true;
        viewer.dirty_grids.extend(edited_grids);
    }

    // Only send updates to peers (and persist to disk) once an edit has been
    // completed, roughly when the user releases the mouse button.
    let pointer_released = ctx.input(|i| i.pointer.any_released());
    if viewer.grids_dirty && pointer_released {
        viewer.grids_dirty = false;
        pending
            .outgoing_broadcast
            .push(NetMsg::Game(GameEvent::UpdateUnitGrids {
                grids: viewer.grids.clone(),
            }));
        save_unit_grids(&viewer.grids);
        // Re-cut only the edited grids' sprites.
        let dirty: Vec<UnitGrid> = viewer
            .dirty_grids
            .iter()
            .filter_map(|&i| viewer.grids.get(i).cloned())
            .collect();
        viewer.dirty_grids.clear();
        cut_sprites_for_grids(&dirty);
    }
}

pub fn unit_grid_labels(
    mut contexts: EguiContexts,
    mode: EditorToolState,
    viewer: Res<UnitViewer>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
    clip: Res<SidebarClip>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    if !mode.is_unit_sheet() {
        return;
    }

    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let Some(vp_size) = camera.logical_viewport_size() else {
        return;
    };

    // Paint into the shared background layer so panels (registered later this frame
    // via SidePanel.show / mode_toolbar Area) append their shapes after ours and
    // visually sit on top.
    let canvas_rect = {
        let screen = ctx.viewport_rect();
        match clip.right_sidebar {
            Some(sidebar) => {
                egui::Rect::from_min_max(screen.min, egui::pos2(sidebar.left(), screen.max.y))
            }
            None => screen,
        }
    };
    let mut painter = ctx.layer_painter(egui::LayerId::background());
    painter.set_clip_rect(canvas_rect);
    let font = egui::FontId::monospace(12.0);
    let char_w = 12.0 * 0.6;
    let line_h = 12.0 * 1.4;
    let padding = egui::vec2(4.0, 1.0);
    for grid in viewer.grids.iter() {
        let world_pos = pixel_to_world(grid.x + grid.width * 0.5, grid.y);
        let Ok(screen) = camera.world_to_viewport(cam_transform, world_pos) else {
            continue;
        };
        if screen.x < 0.0 || screen.x > vp_size.x || screen.y < 0.0 || screen.y > vp_size.y {
            continue;
        }
        let text_w = grid.name.len() as f32 * char_w;
        let center = egui::pos2(screen.x, screen.y - 16.0 + line_h * 0.5);
        let rect = egui::Rect::from_center_size(
            center,
            egui::vec2(text_w + 2.0 * padding.x, line_h + 2.0 * padding.y),
        );
        painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(200));
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            &grid.name,
            font.clone(),
            egui::Color32::WHITE,
        );
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(test)))]
const UNIT_GRIDS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/unit_grids.ron");

#[cfg(all(not(target_arch = "wasm32"), not(test)))]
pub(crate) fn save_unit_grids(grids: &[UnitGrid]) {
    match ron::ser::to_string_pretty(grids, ron::ser::PrettyConfig::default()) {
        Ok(contents) => match std::fs::write(UNIT_GRIDS_PATH, contents) {
            Ok(()) => bevy::log::info!("saved {} unit grids to {UNIT_GRIDS_PATH}", grids.len()),
            Err(e) => bevy::log::error!("failed to write {UNIT_GRIDS_PATH}: {e}"),
        },
        Err(e) => bevy::log::error!("failed to serialize unit grids: {e}"),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn save_unit_grids(_grids: &[UnitGrid]) {}

#[cfg(not(target_arch = "wasm32"))]
fn split_interval(start: f32, len: f32, n: u32) -> Vec<(u32, u32)> {
    let base = (len / n as f32).floor() as u32;
    let extra = len as u32 - base * n;
    let mut offset = start.round() as u32;
    let mut segs = Vec::with_capacity(n as usize);
    for i in 0..n {
        let w = base + if i < extra { 1 } else { 0 };
        segs.push((offset, w));
        offset += w;
    }
    segs
}

#[cfg(not(target_arch = "wasm32"))]
fn clear_sprites_dir() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let out_dir = std::path::Path::new(manifest)
        .join("assets")
        .join("sprites");
    if out_dir.exists() {
        match std::fs::read_dir(&out_dir) {
            Ok(entries) => {
                for entry in entries {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(e) => {
                            warn!("clear_sprites_dir: readdir entry failed: {e}");
                            continue;
                        }
                    };
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "webp")
                        && let Err(e) = std::fs::remove_file(&path)
                    {
                        warn!("clear_sprites_dir: failed to remove {}: {e}", path.display());
                    }
                }
            }
            Err(e) => warn!("clear_sprites_dir: read_dir({}) failed: {e}", out_dir.display()),
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn clear_sprites_dir() {}

/// Cut only `grids` (a subset is fine) out of the units sheet. Opens the source
/// image once; writes one WebP per cell of the given grids. Used to re-cut just
/// the grid(s) the user edited, instead of all of them.
#[cfg(not(target_arch = "wasm32"))]
pub fn cut_sprites_for_grids(grids: &[UnitGrid]) {
    if grids.is_empty() {
        return;
    }
    let src = match image::open(UNITS_SHEET_PATH) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            bevy::log::error!("failed to open {UNITS_SHEET_PATH} for sprite cutting: {e}");
            return;
        }
    };
    let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("sprites");
    let _ = std::fs::create_dir_all(&out_dir);

    let mut total = 0;
    for g in grids {
        let cols = split_interval(g.x, g.width, g.cols);
        let rows = split_interval(g.y, g.height, g.rows);
        for (ri, &(py, ch)) in rows.iter().enumerate() {
            for (ci, &(px, cw)) in cols.iter().enumerate() {
                let cell = image::imageops::crop_imm(&src, px, py, cw, ch).to_image();
                let safe_name = g.name.replace(' ', "_");
                let filename = format!("{}_{}_{}.webp", safe_name, ci, ri);
                if let Err(e) = cell.save(out_dir.join(&filename)) {
                    bevy::log::error!("failed to save sprite {filename}: {e}");
                }
                total += 1;
            }
        }
    }
    bevy::log::info!("cut {total} sprites from {} grid(s)", grids.len());
}

#[cfg(target_arch = "wasm32")]
pub fn cut_sprites_for_grids(_grids: &[UnitGrid]) {}

#[cfg(test)]
mod tests {
    use super::*;

    struct CounterCell {
        unit: String,
        col: u32,
        row: u32,
        rect: (f32, f32, f32, f32),
    }

    fn cut_grids(grids: &[UnitGrid]) -> Vec<CounterCell> {
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
                        rect: (g.x + col as f32 * cw, g.y + row as f32 * ch, cw, ch),
                    });
                }
            }
        }
        cells
    }

    #[test]
    fn cut_grids_from_file() {
        let contents = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/unit_grids.ron"
        ));
        let grids: Vec<UnitGrid> =
            ron::from_str(contents).expect("unit_grids.ron should be valid ron");

        assert!(!grids.is_empty(), "at least one grid");

        let cells = cut_grids(&grids);
        for cell in &cells {
            assert!(cell.rect.0 >= 0.0, "{} col {} x", cell.unit, cell.col);
            assert!(cell.rect.1 >= 0.0, "{} row {} y", cell.unit, cell.row);
            assert!(cell.rect.2 > 0.0, "{} col {} width", cell.unit, cell.col);
            assert!(cell.rect.3 > 0.0, "{} row {} height", cell.unit, cell.row);
            assert!(
                cell.rect.0 + cell.rect.2 <= UNITS_IMG_W + 1.0,
                "{} col {} right edge {} exceeds image width {}",
                cell.unit,
                cell.col,
                cell.rect.0 + cell.rect.2,
                UNITS_IMG_W,
            );
            assert!(
                cell.rect.1 + cell.rect.3 <= UNITS_IMG_H + 1.0,
                "{} row {} bottom edge {} exceeds image height {}",
                cell.unit,
                cell.row,
                cell.rect.1 + cell.rect.3,
                UNITS_IMG_H,
            );
        }

        let expected: usize = grids.iter().map(|g| (g.cols * g.rows) as usize).sum();
        assert_eq!(cells.len(), expected);

        for g in &grids {
            let first = cells
                .iter()
                .find(|c| c.unit == g.name && c.col == 0 && c.row == 0);
            assert!(first.is_some(), "{} missing cell 0,0", g.name);
            let first = first.unwrap();
            assert!((first.rect.0 - g.x).abs() < 0.001, "{} origin x", g.name);
            assert!((first.rect.1 - g.y).abs() < 0.001, "{} origin y", g.name);
            let last = cells
                .iter()
                .find(|c| c.unit == g.name && c.col == g.cols - 1 && c.row == g.rows - 1);
            assert!(
                last.is_some(),
                "{} missing cell {},{}",
                g.name,
                g.cols - 1,
                g.rows - 1
            );
            let last = last.unwrap();
            assert!(
                (last.rect.0 + last.rect.2 - (g.x + g.width)).abs() < 0.001,
                "{} right edge",
                g.name
            );
            assert!(
                (last.rect.1 + last.rect.3 - (g.y + g.height)).abs() < 0.001,
                "{} bottom edge",
                g.name
            );
        }

        println!("-- {} grids, {} cells --", grids.len(), cells.len());
        for g in &grids {
            let count: usize = cells.iter().filter(|c| c.unit == g.name).count();
            println!(
                "  {:<20}  {}x{} = {} cells  @ ({:.0},{:.0}) {}x{}",
                g.name, g.cols, g.rows, count, g.x, g.y, g.width, g.height
            );
        }
    }
}

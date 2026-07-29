use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::editor::LoadedAnnotations;
use crate::state::{AppMode, GameStateResource, GameTurn};

/// Gameplay-mode turn track gizmo: draws the 9×3 boustrophedon grid on the
/// campaign map and highlights the current-turn cell.  Only active for
/// Campaign / Historical scenarios which play on the campaign board.
pub(crate) fn turn_track_gizmos(
    mode: Res<State<AppMode>>,
    game_state: Option<Res<GameStateResource>>,
    turn: Option<Res<GameTurn>>,
    loaded: Res<LoadedAnnotations>,
    mut gizmos: Gizmos,
) {
    if **mode != AppMode::Game {
        return;
    }
    let Some(gs) = game_state else { return };
    let Some(turn) = turn else { return };
    let scenario = gs.0.scenario;
    if !matches!(
        scenario,
        omdurman_types::Scenario::Campaign | omdurman_types::Scenario::Historical
    ) {
        return;
    }
    let map = loaded.map(omdurman_types::MapKind::Campaign);
    let Some(track) = map.campaign_turn_track else {
        return;
    };

    let y = 1.0;
    let cell_w = track.w / 9.0;
    let cell_h = track.h / 3.0;
    let grid_color = Color::srgba(0.35, 0.35, 0.35, 0.6);
    let highlight_color = Color::srgba(1.0, 0.3, 0.2, 0.9);

    let (tl_px, tl_py) = (track.x, track.y);
    let (br_px, br_py) = (track.x + track.w, track.y + track.h);
    let tl = omdurman_hexmap::pixel_to_world_dims(tl_px, tl_py, map.img_w, map.img_h);
    let br = omdurman_hexmap::pixel_to_world_dims(br_px, br_py, map.img_w, map.img_h);
    let left = tl.x;
    let right = br.x;
    let top = tl.z;
    let bottom = br.z;

    // Vertical grid lines (cols 1..8).
    for c in 1..9 {
        let cx_px = track.x + c as f32 * cell_w;
        let cx =
            omdurman_hexmap::pixel_to_world_dims(cx_px, tl_py, map.img_w, map.img_h).x;
        gizmos.line(Vec3::new(cx, y, top), Vec3::new(cx, y, bottom), grid_color);
    }
    // Horizontal grid lines (rows 1..2).
    for r in 1..3 {
        let cy_px = track.y + r as f32 * cell_h;
        let cz =
            omdurman_hexmap::pixel_to_world_dims(tl_px, cy_px, map.img_w, map.img_h).z;
        gizmos.line(Vec3::new(left, y, cz), Vec3::new(right, y, cz), grid_color);
    }

    // Highlight the current-turn cell.
    let idx = (**turn as usize).saturating_sub(1);
    let row = idx / 9;
    let col = idx % 9;
    if row < 3 {
        let n_cols = if row == 2 { 4 } else { 9 };
        if col < n_cols {
            let cell_left_px = match row {
                0 | 2 => track.x + col as f32 * cell_w,
                1 => track.x + (9.0_f32 - col as f32 - 1.0) * cell_w,
                _ => return,
            };
            let cell_right_px = cell_left_px + cell_w;
            let cell_top_px = track.y + row as f32 * cell_h;
            let cell_bottom_px = cell_top_px + cell_h;

            let cl = omdurman_hexmap::pixel_to_world_dims(
                cell_left_px,
                cell_top_px,
                map.img_w,
                map.img_h,
            );
            let cr = omdurman_hexmap::pixel_to_world_dims(
                cell_right_px,
                cell_bottom_px,
                map.img_w,
                map.img_h,
            );

            let (hx, hz) = (cl.x, cl.z);
            let (hx2, hz2) = (cr.x, cr.z);

            gizmos.line(Vec3::new(hx, y, hz), Vec3::new(hx2, y, hz), highlight_color);
            gizmos.line(Vec3::new(hx2, y, hz), Vec3::new(hx2, y, hz2), highlight_color);
            gizmos.line(Vec3::new(hx2, y, hz2), Vec3::new(hx, y, hz2), highlight_color);
            gizmos.line(Vec3::new(hx, y, hz2), Vec3::new(hx, y, hz), highlight_color);
        }
    }
}

/// Gameplay-mode turn track cell labels: renders date/time text at each grid
/// cell centre, projected from 3D world space to screen coordinates.
pub(crate) fn turn_track_labels(
    mut contexts: EguiContexts,
    mode: Res<State<AppMode>>,
    game_state: Option<Res<GameStateResource>>,
    turn: Option<Res<GameTurn>>,
    loaded: Res<LoadedAnnotations>,
    cameras: Query<(&Camera, &GlobalTransform), With<crate::camera::RtsCamera>>,
) {
    if **mode != AppMode::Game {
        return;
    }
    let Some(gs) = game_state else { return };
    let Some(turn) = turn else { return };
    let scenario = gs.0.scenario;
    if !matches!(
        scenario,
        omdurman_types::Scenario::Campaign | omdurman_types::Scenario::Historical
    ) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };
    let map = loaded.map(omdurman_types::MapKind::Campaign);
    let Some(track) = map.campaign_turn_track else {
        return;
    };

    let cell_w = track.w / 9.0;
    let cell_h = track.h / 3.0;

    let screen_centre = |px: f32, py: f32| -> Option<egui::Pos2> {
        let world = omdurman_hexmap::pixel_to_world_dims(px, py, map.img_w, map.img_h);
        let world_pos = Vec3::new(world.x, 0.0, world.z);
        let viewport = camera.world_to_viewport(cam_transform, world_pos).ok()?;
        Some(egui::pos2(viewport.x, viewport.y))
    };

    let current_idx = (**turn as usize).saturating_sub(1);

    for row in 0..3u8 {
        let n_cols = if row == 2 { 4 } else { 9u8 };
        for col in 0..n_cols {
            let idx = row * 9 + col;
            let turn_num = idx + 1;
            let label = omdurman_rules::turn_track::TurnLabel::from_turn(turn_num);

            let cx_px = match row {
                0 | 2 => track.x + (col as f32 + 0.5) * cell_w,
                1 => track.x + (9.0_f32 - col as f32 - 0.5) * cell_w,
                _ => unreachable!(),
            };
            let cy_px = track.y + (row as f32 + 0.5) * cell_h;

            let Some(screen) = screen_centre(cx_px, cy_px) else {
                continue;
            };

            let text = match label {
                Some(l) => l.to_string(),
                None => format!("Turn {turn_num}"),
            };

            let is_current = (idx as usize) == current_idx;
            let (color, size) = if is_current {
                (
                    egui::Color32::from_rgba_premultiplied(255, 100, 80, 240),
                    11.0,
                )
            } else {
                (
                    egui::Color32::from_rgba_premultiplied(180, 180, 180, 140),
                    9.0,
                )
            };

            ctx.debug_painter().text(
                screen,
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId {
                    size,
                    family: egui::FontFamily::Monospace,
                },
                color,
            );
        }
    }
}

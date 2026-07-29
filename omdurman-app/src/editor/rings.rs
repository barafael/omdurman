//! Editor-only hex ring overlays: excluded-hex outlines (red) and selection
//! highlights (green). Cache-keyed to avoid per-frame entity churn.
//! Each spawned ring carries [`DespawnOnExit`] for both [`EditorTab::Terrain`]
//! and [`AppMode::Editor`], so manual OnExit cleanup systems are unnecessary.

use bevy::prelude::*;
use omdurman_hexmap::{GameMap, HexLayout, hex_world_pos};
use omdurman_types::HexCoord;

use crate::render::{HexOverlay, HexRingAssets};
use crate::ui::despawn_all;
use crate::{AppMode, EditorTab};

use super::HexEditor;

/// Draw excluded hexes with a red outline while in Editor mode, so the holes in
/// the map (board furniture) are visible during terrain editing.
#[derive(Component)]
pub(crate) struct ExcludedHexRing;

pub(super) fn draw_excluded_hex_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    game_map: Res<GameMap>,
    existing: Query<Entity, With<ExcludedHexRing>>,
    mut last: Local<Vec<HexCoord>>,
) {
    let mut current: Vec<HexCoord> = game_map.excluded.iter().copied().collect();
    current.sort_by_key(|c| (c.q, c.r));
    if &current == &*last {
        return;
    }
    let existing: Vec<Entity> = existing.iter().collect();
    despawn_all(&mut commands, &existing);
    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for coord in &game_map.excluded {
        let pos = hex_world_pos(*coord, origin, &overlay.params);
        commands.spawn((
            ExcludedHexRing,
            DespawnOnExit(EditorTab::Terrain),
            DespawnOnExit(AppMode::Editor),
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.red.clone()),
            Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
    *last = current;
}

/// Draw selected hexes with green outlines in Editor mode. The anchor hex
/// (whose state the panel shows) gets a brighter shade.
#[derive(Component)]
pub(crate) struct EditorHighlightRing;

pub(super) fn draw_editor_highlight_mesh(
    mut commands: Commands,
    assets: Res<HexRingAssets>,
    layout: Res<HexLayout>,
    overlay: Res<HexOverlay>,
    editor: Res<HexEditor>,
    existing: Query<Entity, With<EditorHighlightRing>>,
    mut last: Local<(Vec<HexCoord>, Option<HexCoord>)>,
) {
    let mut current_sel: Vec<HexCoord> = editor.selection.iter().copied().collect();
    current_sel.sort_by_key(|c| (c.q, c.r));
    let current = (current_sel, editor.anchor);
    if &current == &*last {
        return;
    }
    let existing: Vec<Entity> = existing.iter().collect();
    despawn_all(&mut commands, &existing);
    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for &coord in &editor.selection {
        let pos = hex_world_pos(coord, origin, &overlay.params);
        let is_anchor = editor.anchor == Some(coord);
        let s = if is_anchor { size } else { size * 0.92 };
        commands.spawn((
            EditorHighlightRing,
            DespawnOnExit(EditorTab::Terrain),
            DespawnOnExit(AppMode::Editor),
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(if is_anchor {
                assets.light_green.clone()
            } else {
                assets.green.clone()
            }),
            Transform::from_xyz(pos.x, 1.5, pos.z).with_scale(Vec3::splat(s)),
            Visibility::Visible,
        ));
    }
    *last = current;
}

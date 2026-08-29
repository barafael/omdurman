//! Line-of-sight overlay (§6.3): with the toggle on, hovering a hex marks
//! every hex within maximum fire range as clear (green ring) or blocked
//! (dark red ring) for a ground-level direct-fire shot from the hovered hex.
//!
//! The analysis uses the engine's LOS table ([`omdurman_rules::los_table`])
//! against the *current* board -- live game or the scrubbed spectator state,
//! whichever feeds [`GameStateResource`].

use std::collections::HashSet;

use bevy::prelude::*;
use omdurman_hexmap::hex_world_pos;
use omdurman_rules::los_table::{LosLevel, LosStepResult, los_path_analysis};
use omdurman_types::HexCoord;

use crate::GameStateResource;

/// Marker for one LOS overlay ring.
#[derive(Component)]
pub(crate) struct LosRing;

/// Runtime toggle for the LOS overlay, flipped by the toolbar button next to
/// ZOC.
#[derive(Resource, Default)]
pub struct LosOverlay {
    pub visible: bool,
}

/// Maximum direct-fire range on either board's range-effects table (§6.11):
/// the overlay tints everything that could conceivably be in range.
const MAX_RANGE: u32 = 10;

/// Spawn green/red rings for LOS from the hovered hex. Rebuilt only when the
/// hovered hex (or the underlying board) changes.
pub fn los_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    toggle: Res<LosOverlay>,
    hovered: Res<crate::render::HoveredHex>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<LosRing>>,
    mut last: Local<Option<HashSet<HexCoord>>>,
) {
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let existing: Vec<Entity> = existing.iter().collect();

    let Some(from) = (toggle.visible).then(|| hovered.0).flatten() else {
        if !existing.is_empty() {
            crate::ui::despawn_all(&mut commands, &existing);
            *last = None;
        }
        return;
    };
    let Some(gs) = game_state else { return };

    // Rebuild when the hexes in LOS from `from` differ from last time (the
    // hovered hex moved, or the board changed under us).
    let los = los_from(&gs.0, from);
    if last.as_ref() == Some(&los.blocked) && existing.len() == los.blocked.len() + los.clear.len()
    {
        return;
    }
    crate::ui::despawn_all(&mut commands, &existing);

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for hex in &los.clear {
        let pos = hex_world_pos(*hex, origin, &overlay.params);
        commands.spawn((
            LosRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.light_green.clone()),
            Transform::from_xyz(pos.x, 1.45, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
    for hex in &los.blocked {
        let pos = hex_world_pos(*hex, origin, &overlay.params);
        commands.spawn((
            LosRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.marker_red.clone()),
            Transform::from_xyz(pos.x, 1.45, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
}

/// The LOS partition from `from`: which hexes in range are clear vs blocked
/// for a ground-level direct shot (§6.3). Board-only analysis -- intervening
/// *units* on hilltops are not considered (that is firing-unit dependent).
struct LosPartition {
    clear: HashSet<HexCoord>,
    blocked: HashSet<HexCoord>,
}

fn los_from(gs: &omdurman_rules::effects::GameState, from: HexCoord) -> LosPartition {
    let mut clear = HashSet::new();
    let mut blocked = HashSet::new();
    if gs.board.terrain_at(from).is_none() {
        return LosPartition { clear, blocked };
    }
    for dq in -(MAX_RANGE as i32)..=(MAX_RANGE as i32) {
        for dr in -(MAX_RANGE as i32)..=(MAX_RANGE as i32) {
            let to = HexCoord::new(from.q + dq, from.r + dr);
            if to == from || from.distance(to) > MAX_RANGE || gs.board.terrain_at(to).is_none() {
                continue;
            }
            let steps = los_path_analysis(
                &gs.board,
                from,
                to,
                omdurman_rules::FireKind::Direct,
                LosLevel::Ground,
                LosLevel::Ground,
                |_| None,
            );
            let is_blocked = steps.iter().any(|(_, r)| {
                matches!(
                    r,
                    LosStepResult::Blocked { .. } | LosStepResult::BlockedHexside { .. }
                )
            });
            if is_blocked {
                blocked.insert(to);
            } else {
                clear.insert(to);
            }
        }
    }
    LosPartition { clear, blocked }
}

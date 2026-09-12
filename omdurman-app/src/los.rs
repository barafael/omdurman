//! Line-of-sight overlay (§6.3): with the toggle on, hovering a hex marks
//! every hex within maximum fire range as clear (green ring) or blocked
//! (dark red ring) for a ground-level direct-fire shot from that hex.
//! During a fire sub-phase the origin is instead the *selected firing
//! position* (the fire selection's hex), and every blocked hex carries an
//! on-map label naming the blocking feature ("trees", "wall", ...) so the
//! player can see *why* a target hex is out of sight.
//!
//! The analysis uses the engine's LOS table ([`omdurman_rules::los_table`])
//! against the *current* board -- live game or the scrubbed spectator state,
//! whichever feeds [`GameStateResource`].

use std::collections::HashSet;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hexmap::hex_world_pos;
use omdurman_rules::los_table::{LosFeature, LosLevel, LosStepResult, los_path_analysis};
use omdurman_rules::{FireKind, Phase};
use omdurman_types::HexCoord;

use crate::GameStateResource;
use crate::fire::fire_selection;
use crate::picker::{HexMapView, PickerState, PlacedUnit};

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

/// Cached LOS partition. Computed by [`update_los_analysis`] only when the
/// origin hex changes (the origin is the hovered hex, or the selected firing
/// hex during a fire sub-phase), then consumed by both [`los_overlay_mesh`]
/// (rings) and [`los_blocked_labels`] (reason labels) so the partition is
/// computed at most once per origin instead of once per consumer per frame.
#[derive(Resource, Default)]
pub(crate) struct LosAnalysis {
    pub from: Option<HexCoord>,
    pub clear: Vec<HexCoord>,
    /// Blocked hexes with the label(s) explaining what blocks (§6.3).
    pub blocked: Vec<(HexCoord, String)>,
}

/// Recommpute the LOS partition for the active origin: the hovered hex, or --
/// during a fire sub-phase -- the *selected firing position* (so the player
/// sees exactly what their group can see before picking targets, §6.3/§6.41).
pub fn update_los_analysis(
    toggle: Res<LosOverlay>,
    hovered: Res<crate::render::HoveredHex>,
    picker: Res<PickerState>,
    placed_units: Query<(Entity, &PlacedUnit)>,
    game_state: Option<Res<GameStateResource>>,
    mut analysis: ResMut<LosAnalysis>,
) {
    let hovered_from = hovered.0;
    let from = match game_state.as_deref() {
        Some(gs)
            if matches!(
                gs.0.phase,
                Phase::OffensiveFire(_) | Phase::DefensiveFire(_)
            ) =>
        {
            fire_selection(&picker, &placed_units, &gs.0).map(|g| g.firer_hex)
        }
        _ => hovered_from,
    };
    if !toggle.visible {
        analysis.from = None;
        analysis.clear.clear();
        analysis.blocked.clear();
        return;
    }
    let Some(from) = from else {
        analysis.from = None;
        analysis.clear.clear();
        analysis.blocked.clear();
        return;
    };
    if analysis.from == Some(from) {
        return;
    }
    let Some(gs) = game_state else {
        analysis.from = None;
        analysis.clear.clear();
        analysis.blocked.clear();
        return;
    };
    let partition = los_from(&gs.0, from);
    analysis.from = Some(from);
    analysis.clear = partition.clear.into_iter().collect();
    analysis.blocked = partition.blocked;
}

/// Spawn green/red rings for LOS from the analysis origin. Rebuilt only when
/// the origin hex changes.
pub fn los_overlay_mesh(
    mut commands: Commands,
    hex: crate::HexRender,
    analysis: Res<LosAnalysis>,
    existing: Query<Entity, With<LosRing>>,
    mut last: Local<Option<HexCoord>>,
) {
    let crate::HexRender {
        assets,
        layout,
        overlay,
    } = hex;
    let existing: Vec<Entity> = existing.iter().collect();

    let Some(from) = analysis.from else {
        if !existing.is_empty() {
            crate::ui::despawn_all(&mut commands, &existing);
            *last = None;
        }
        return;
    };
    if *last == Some(from) {
        return;
    }
    crate::ui::despawn_all(&mut commands, &existing);

    let origin = layout.adjusted_origin(&overlay.params);
    let size = overlay.params.hex_size;
    for hex in &analysis.clear {
        let pos = hex_world_pos(*hex, origin, &overlay.params);
        commands.spawn((
            LosRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.light_green.clone()),
            Transform::from_xyz(pos.x, 1.45, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
    for (hex, _) in &analysis.blocked {
        let pos = hex_world_pos(*hex, origin, &overlay.params);
        commands.spawn((
            LosRing,
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.marker_red.clone()),
            Transform::from_xyz(pos.x, 1.45, pos.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
    *last = Some(from);
}

/// On-map labels for every hex the LOS partition marks as blocked: a small
/// dark-red tag naming the blocking feature(s) ("trees", "wall", ... §6.3),
/// projected to screen space. Drawn directly over the blocked ring so the
/// player reads *why* a target is out of sight without opening the rulebook.
pub fn los_blocked_labels(
    mut contexts: EguiContexts,
    analysis: Res<LosAnalysis>,
    view: HexMapView,
) {
    let Some(_from) = analysis.from else {
        return;
    };
    if analysis.blocked.is_empty() {
        return;
    }
    let HexMapView {
        layout,
        overlay,
        cameras,
        ..
    } = view;
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };

    // Dim the origin ring label differently: the origin isn't "blocked", it
    // is the viewing point.
    let origin = layout.adjusted_origin(&overlay.params);
    for (hex, reason) in &analysis.blocked {
        let pos = hex_world_pos(*hex, origin, &overlay.params);
        let world_pos_3d = Vec3::new(pos.x, 2.2, pos.z);
        let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos_3d) else {
            continue;
        };
        egui::Area::new(egui::Id::new(("los_blocked_label", hex)))
            .fixed_pos(egui::pos2(screen_pos.x - 8.0, screen_pos.y + 14.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_premultiplied(40, 10, 10, 200))
                    .corner_radius(3.0)
                    .inner_margin(egui::Margin::symmetric(5, 2))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(reason)
                                .color(egui::Color32::from_rgb(235, 150, 130))
                                .size(11.0)
                                .strong(),
                        );
                    });
            });
    }
}

/// The LOS partition from `from`: which hexes in range are clear vs blocked
/// for a ground-level direct shot (§6.3). Board-only analysis -- intervening
/// *units* on hilltops are not considered (that is firing-unit dependent).
struct LosPartition {
    clear: HashSet<HexCoord>,
    blocked: Vec<(HexCoord, String)>,
}

/// Short on-map label for a blocking feature (§6.3).
fn feature_label(f: LosFeature) -> &'static str {
    match f {
        LosFeature::Units => "units",
        LosFeature::Huts => "huts",
        LosFeature::Wall => "wall",
        LosFeature::Trees => "trees",
        LosFeature::Crest => "crest",
        LosFeature::RoughTerrain => "rough",
        LosFeature::HilltopTerrain => "hilltop",
    }
}

fn los_from(gs: &omdurman_rules::effects::GameState, from: HexCoord) -> LosPartition {
    let mut clear = HashSet::new();
    let mut blocked: Vec<(HexCoord, String)> = Vec::new();
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
                FireKind::Direct,
                LosLevel::Ground,
                LosLevel::Ground,
                |_| None,
                |a, b| gs.wall_is_breached(a, b),
            );
            let mut blocking: Vec<LosFeature> = Vec::new();
            for (_, r) in &steps {
                let f = match r {
                    LosStepResult::Blocked { feature, .. } => Some(*feature),
                    LosStepResult::BlockedHexside { feature, .. } => Some(*feature),
                    LosStepResult::Clear => None,
                };
                if let Some(f) = f
                    && !blocking.contains(&f)
                {
                    blocking.push(f);
                }
            }
            if blocking.is_empty() {
                clear.insert(to);
            } else {
                let reason = blocking
                    .iter()
                    .take(2)
                    .map(|f| feature_label(*f))
                    .collect::<Vec<&str>>()
                    .join(" · ");
                blocked.push((to, reason));
            }
        }
    }
    LosPartition { clear, blocked }
}

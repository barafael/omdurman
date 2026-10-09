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
    // Recompute when the origin moves or the game changes (units block,
    // §6.3 "Units").
    if analysis.from == Some(from) && !game_state.as_ref().is_some_and(|gs| gs.is_changed()) {
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
    (generation, mut seen_generation): (Res<crate::picker::OverlayGeneration>, Local<u32>),
) {
    if generation.invalidates(&mut seen_generation) {
        *last = None;
    }
    let existing: Vec<Entity> = existing.iter().collect();

    let Some(from) = analysis.from else {
        if !existing.is_empty() {
            crate::ui::despawn_all(&mut commands, &existing);
            *last = None;
        }
        return;
    };
    if *last == Some(from) && !analysis.is_changed() {
        return;
    }
    crate::ui::despawn_all(&mut commands, &existing);

    // Just under the movement-path arrows (1.45): at their height the two
    // shared depth wherever an arrow crossed a ring and flickered.
    let mut rings = crate::overlay::ring_batch(&mut commands, &hex, existing);
    for target in &analysis.clear {
        rings.ring(LosRing, *target, 1.43, 1.0, &hex.assets.light_green);
    }
    for (target, _) in &analysis.blocked {
        rings.ring(LosRing, *target, 1.43, 1.0, &hex.assets.marker_red);
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
    let screen = |hex: HexCoord| {
        let pos = hex_world_pos(hex, origin, &overlay.params);
        camera
            .world_to_viewport(camera_transform, Vec3::new(pos.x, 2.2, pos.z))
            .ok()
    };
    // The on-screen hex size (centre to neighbouring centre) sets the label
    // size; zoomed far out the tags no longer fit their hexes and the board
    // turns into a wall of them, so they go (the ring tint stays).
    let Some(&(probe, _)) = analysis.blocked.first() else {
        return;
    };
    let hex_px = match (screen(probe), screen(HexCoord::new(probe.q + 1, probe.r))) {
        (Some(a), Some(b)) => a.distance(b),
        _ => return,
    };
    let Some(size) = los_label_size(hex_px) else {
        return;
    };
    for (hex, reason) in &analysis.blocked {
        let Some(screen_pos) = screen(*hex) else {
            continue;
        };
        egui::Area::new(egui::Id::new(("los_blocked_label", hex)))
            .fixed_pos(egui::pos2(screen_pos.x, screen_pos.y + hex_px * 0.18))
            .pivot(egui::Align2::CENTER_TOP)
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                crate::ui::frames::tag(crate::ui::palette::REFUSAL_TAG_BG, 5).show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(reason)
                                .color(crate::ui::palette::ALERT)
                                .size(size)
                                .strong(),
                        )
                        .extend(),
                    );
                });
            });
    }
}

/// The font size for the LOS reason tags at `hex_px` screen pixels between
/// neighbouring hex centres, or `None` when a tag ("hilltop", ~7 characters
/// plus padding) would no longer fit inside its hex.
fn los_label_size(hex_px: f32) -> Option<f32> {
    let size = ((0.9 * hex_px - 10.0) / 3.9).min(11.0);
    (size >= 7.0).then_some(size)
}

/// The LOS partition from `from`: which hexes in range are clear vs blocked
/// for a direct shot (§6.3), as the engine sees it -- the firer's and each
/// target's level (a unit's own, §6.3 notes b/c, else its hex's terrain) and
/// the units in between (not gunboats, forts or entrenched units).
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
    let level_at = |hex: HexCoord| {
        gs.units.iter().find(|u| u.position == hex).map_or_else(
            || {
                gs.board
                    .terrain_at(hex)
                    .map_or(LosLevel::Ground, omdurman_rules::los_table::los_level)
            },
            |u| omdurman_rules::los_table::los_level_for_unit(u.profile.kind, hex, &gs.board),
        )
    };
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
                level_at(from),
                level_at(to),
                gs.los_unit_blocker(),
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

#[cfg(test)]
mod tests {
    use super::los_label_size;

    /// Play-test repro: zoomed out, "hilltop" tags overflowed and wrapped
    /// ("hillt/op") across the board. Tags shrink with the hexes and vanish
    /// once they no longer fit.
    #[test]
    fn los_tags_scale_with_the_hexes_and_hide_when_too_small() {
        assert_eq!(
            los_label_size(120.0),
            Some(11.0),
            "capped at the normal size"
        );
        let mid = los_label_size(50.0).expect("a 50 px hex still carries a tag");
        assert!((7.0..11.0).contains(&mid), "{mid}");
        assert_eq!(los_label_size(30.0), None, "too small to carry a tag");
    }
}

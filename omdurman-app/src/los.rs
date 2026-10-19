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
use omdurman_rules::FireKind;
use omdurman_rules::los_table::{LosFeature, LosStepResult, los_path_analysis};
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
    /// The origin pinned with P: the overlay reads from it whatever is
    /// hovered, and the ray to the hovered hex is drawn.
    pub pinned: Option<HexCoord>,
}

/// Marker for one segment of the drawn line of sight.
#[derive(Component)]
pub(crate) struct LosRayBar;

/// With the LOS overlay on, P pins the hovered hex as the origin (again on
/// the same hex: unpins). Turning the overlay off drops the pin.
pub fn pin_los_origin(
    keys: Res<ButtonInput<KeyCode>>,
    hovered: Res<crate::render::HoveredHex>,
    mut toggle: ResMut<LosOverlay>,
) {
    if !toggle.visible {
        if toggle.pinned.is_some() {
            toggle.pinned = None;
        }
        return;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        toggle.pinned = match (toggle.pinned, hovered.0) {
            (Some(pinned), Some(at)) if pinned == at => None,
            (_, Some(at)) => Some(at),
            (pinned, None) => pinned,
        };
    }
}

/// The line of sight from the pinned origin to the hovered hex (§6.3):
/// green as far as it runs clear, red from the feature that blocks it on
/// (the blocked hexes' labels name it). Rebuilt when either end or the
/// game changes.
#[allow(clippy::too_many_arguments)]
pub fn los_ray(
    mut commands: Commands,
    hex: crate::HexRender,
    toggle: Res<LosOverlay>,
    hovered: Res<crate::render::HoveredHex>,
    game_state: Option<Res<GameStateResource>>,
    existing: Query<Entity, With<LosRayBar>>,
    mut last: Local<Option<(HexCoord, HexCoord)>>,
    (generation, mut seen_generation): (Res<crate::picker::OverlayGeneration>, Local<u32>),
) {
    if generation.invalidates(&mut seen_generation) {
        *last = None;
    }
    let ends = toggle
        .pinned
        .filter(|_| toggle.visible)
        .zip(hovered.0)
        .filter(|(from, to)| from != to);
    let moved = game_state.as_ref().is_some_and(|gs| gs.is_changed());
    if ends == *last && !moved {
        return;
    }
    let mut bars = crate::overlay::ring_batch(&mut commands, &hex, existing.iter());
    *last = ends;
    let (Some((from, to)), Some(gs)) = (ends, game_state.as_deref()) else {
        return;
    };
    let gs = &gs.0;
    let segments = los_segments(gs, from, to);
    let origin = bars.origin();
    let params = &hex.overlay.params;
    let size = bars.size();
    for ((a, b), clear) in segments {
        let pa = hex_world_pos(a, origin, params);
        let pb = hex_world_pos(b, origin, params);
        let mid = (pa + pb) * 0.5;
        let len = pa.distance(pb).max(0.001);
        let dir = (pb - pa) / len;
        let angle = (-dir.z).atan2(dir.x);
        bars.commands().spawn((
            LosRayBar,
            Mesh3d(hex.assets.unit_square.clone()),
            MeshMaterial3d(if clear {
                hex.assets.green.clone()
            } else {
                hex.assets.red.clone()
            }),
            Transform::from_translation(Vec3::new(mid.x, 1.7, mid.z))
                .with_rotation(
                    Quat::from_rotation_y(angle)
                        * Quat::from_rotation_x(-std::f32::consts::PI / 2.0),
                )
                .with_scale(Vec3::new(len, size * 0.08, 1.0)),
            Visibility::Visible,
        ));
    }
}

/// The line from `from` to `to` (§6.3) hex to hex, each step marked clear
/// until the feature that blocks it: the engine's ray analysis, a ground
/// level shot as the overlay reads.
pub(crate) fn los_segments(
    gs: &omdurman_rules::effects::GameState,
    from: HexCoord,
    to: HexCoord,
) -> Vec<((HexCoord, HexCoord), bool)> {
    let steps = los_path_analysis(
        gs.board,
        from,
        to,
        FireKind::Direct,
        gs.target_los_level(from),
        gs.target_los_level(to),
        gs.los_unit_blocker(),
        |a, b| gs.wall_is_breached(a, b),
    );
    // Where the sight ends: the blocking hex, or the near side of a blocking
    // hexside.
    let block = steps.iter().find_map(|(_, r)| match r {
        LosStepResult::Blocked { hex, .. } => Some(*hex),
        LosStepResult::BlockedHexside { a, .. } => Some(*a),
        LosStepResult::Clear => None,
    });
    let line: Vec<HexCoord> = steps
        .iter()
        .map(|(h, _)| *h)
        .chain(
            omdurman_rules::los_table::los_ray_sides(from, to)[0]
                .iter()
                .copied(),
        )
        .fold(Vec::new(), |mut line, h| {
            if !line.contains(&h) {
                line.push(h);
            }
            line
        });
    let mut clear = true;
    line.windows(2)
        .map(|pair| {
            let segment = ((pair[0], pair[1]), clear);
            if Some(pair[1]) == block || Some(pair[0]) == block {
                clear = false;
            }
            segment
        })
        .collect()
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
    // A pinned origin (P) wins over the hovered hex and the firing group.
    let from = match game_state.as_deref() {
        _ if toggle.pinned.is_some() => toggle.pinned,
        Some(gs) if gs.0.phase.is_fire() => {
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
    let from_level = gs.target_los_level(from);
    // Built once: the blocker indexes the board's units when it is made.
    let unit_blocker = gs.los_unit_blocker();
    for dq in -(MAX_RANGE as i32)..=(MAX_RANGE as i32) {
        for dr in -(MAX_RANGE as i32)..=(MAX_RANGE as i32) {
            let to = HexCoord::new(from.q + dq, from.r + dr);
            if to == from || from.distance(to) > MAX_RANGE || gs.board.terrain_at(to).is_none() {
                continue;
            }
            let steps = los_path_analysis(
                gs.board,
                from,
                to,
                FireKind::Direct,
                from_level,
                gs.target_los_level(to),
                &unit_blocker,
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

#[cfg(test)]
mod ray_tests {
    use omdurman_rules::{UnitId, UnitPlacement};
    use omdurman_types::{HexCoord, Scenario};

    /// §6.3: on open ground the line runs clear; a unit in its middle
    /// blocks it from there on.
    #[test]
    fn a_unit_on_the_line_blocks_it_from_there_on() {
        let mut gs = omdurman_rules::effects::GameState::new(Scenario::Campaign);
        gs.board = omdurman_rules::board_data::board_info(omdurman_types::MapKind::Campaign);
        let (from, to) = (HexCoord::new(20, 20), HexCoord::new(23, 20));
        let open = super::los_segments(&gs, from, to);
        assert_eq!(open.len(), 3, "{open:?}");
        assert!(open.iter().all(|(_, clear)| *clear), "{open:?}");
        gs.units.push(UnitPlacement {
            id: UnitId::Baggara_0_0,
            position: HexCoord::new(21, 20),
            profile: omdurman_rules::unit_profiles::profile_for_unit(UnitId::Baggara_0_0).unwrap(),
            state: Default::default(),
        });
        let blocked = super::los_segments(&gs, from, to);
        assert!(blocked[0].1, "the sight reaches the unit");
        assert!(blocked[1..].iter().all(|(_, clear)| !*clear), "{blocked:?}");
    }
}

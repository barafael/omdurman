//! Leading the eye to the right hex: a small pointer at the board's edge for
//! a recent sighting (or the hovered combat card's hexes) that lies out of
//! view, and rings on the hovered combat card's hexes.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use omdurman_hexmap::hex_world_pos;
use omdurman_types::HexCoord;

use super::MotionSettings;
use super::transient::{FX_HEIGHT, FxAssets};
use crate::camera::{RtsCamera, RtsCameraState};

/// How long a sighting keeps its edge pointer (seconds).
const SIGHTING_TTL: f32 = 4.0;
/// Fade-out at the end of a sighting's life (seconds).
const SIGHTING_FADE: f32 = 1.0;
/// At most this many sightings are pointed at: the most recent.
const MAX_POINTERS: usize = 3;

/// Recent hexes where something happened that the local player did not do.
#[derive(Resource, Default)]
pub struct Sightings(Vec<Sighting>);

struct Sighting {
    hex: HexCoord,
    age: f32,
    /// The follow-camera already considered it.
    followed: bool,
}

impl Sightings {
    pub fn see(&mut self, hex: HexCoord) {
        self.0.retain(|s| s.hex != hex);
        self.0.push(Sighting {
            hex,
            age: 0.0,
            followed: false,
        });
        if self.0.len() > MAX_POINTERS {
            self.0.remove(0);
        }
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// The hexes of the combat card under the pointer (written by the event
/// feed each frame): ringed on the board, and pointed at when out of view.
#[derive(Resource, Default, PartialEq, Eq)]
pub struct CardFocus(pub Vec<HexCoord>);

#[derive(Component)]
pub(super) struct CardFocusRing;

/// Ring the hovered combat card's hexes (rebuilt only when they change).
pub(super) fn card_focus_rings(
    focus: Res<CardFocus>,
    assets: Option<Res<FxAssets>>,
    board: crate::BoardGeometry,
    existing: Query<Entity, With<CardFocusRing>>,
    mut commands: Commands,
) {
    if !focus.is_changed() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let Some(assets) = assets else { return };
    let origin = board.layout.adjusted_origin(&board.overlay.params);
    let size = board.overlay.params.hex_size;
    for &hex in &focus.0 {
        let p = hex_world_pos(hex, origin, &board.overlay.params);
        commands.spawn((
            CardFocusRing,
            Mesh3d(assets.ring.clone()),
            MeshMaterial3d(assets.focus.clone()),
            Transform::from_xyz(p.x, FX_HEIGHT, p.z).with_scale(Vec3::splat(size)),
            Visibility::Visible,
        ));
    }
}

/// Where a pointer at `target` (screen space) sits on the edge of `board`,
/// inset by `margin`, and the direction it points. `None` when the target is
/// on the board already.
pub fn edge_anchor(
    board: egui::Rect,
    target: egui::Pos2,
    margin: f32,
) -> Option<(egui::Pos2, egui::Vec2)> {
    if board.contains(target) {
        return None;
    }
    let inner = board.shrink(margin);
    if inner.width() <= 0.0 || inner.height() <= 0.0 {
        return None;
    }
    let center = inner.center();
    let dir = target - center;
    if dir.length_sq() < f32::EPSILON {
        return None;
    }
    // Scale the ray from the centre so it just touches the inner rect.
    let tx = if dir.x.abs() > f32::EPSILON {
        (inner.width() * 0.5) / dir.x.abs()
    } else {
        f32::INFINITY
    };
    let ty = if dir.y.abs() > f32::EPSILON {
        (inner.height() * 0.5) / dir.y.abs()
    } else {
        f32::INFINITY
    };
    Some((center + dir * tx.min(ty), dir.normalized()))
}

/// A small pointer at the board's edge for each sighting (and the hovered
/// card's hexes) out of view; a click pans the camera there. With
/// [`MotionSettings::follow_opponent`], a fresh sighting out of view pans by
/// itself.
#[allow(clippy::too_many_arguments)]
pub(super) fn offscreen_pointers_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    settings: Res<MotionSettings>,
    mut sightings: ResMut<Sightings>,
    focus: Res<CardFocus>,
    board_geometry: crate::BoardGeometry,
    layout: Res<crate::ScreenLayout>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut RtsCameraState), With<RtsCamera>>,
) {
    let dt = time.delta_secs();
    for sighting in &mut sightings.0 {
        sighting.age += dt;
    }
    sightings.0.retain(|s| s.age < SIGHTING_TTL);
    if sightings.0.is_empty() && focus.0.is_empty() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform, mut cam_state)) = cameras.single_mut() else {
        return;
    };
    let screen = ctx.content_rect();
    let board = egui::Rect::from_min_max(
        egui::pos2(layout.left_inset, layout.top_bar_height),
        egui::pos2(screen.right() - layout.right_inset, screen.bottom()),
    );
    let origin = board_geometry
        .layout
        .adjusted_origin(&board_geometry.overlay.params);
    let world = |hex: HexCoord| hex_world_pos(hex, origin, &board_geometry.overlay.params);
    let on_screen = |p: Vec3| {
        camera
            .world_to_viewport(cam_transform, p)
            .ok()
            .map(|v| egui::pos2(v.x, v.y))
    };

    // (target hex, opacity) for every pointer candidate, newest first.
    let mut targets: Vec<(HexCoord, f32)> = focus.0.iter().map(|&hex| (hex, 1.0)).collect();
    let mut pan_to: Option<Vec3> = None;
    for sighting in sightings.0.iter_mut().rev() {
        let alpha = ((SIGHTING_TTL - sighting.age) / SIGHTING_FADE).clamp(0.0, 1.0);
        let p = world(sighting.hex);
        let visible = on_screen(p).is_some_and(|s| board.contains(s));
        if !sighting.followed {
            sighting.followed = true;
            if settings.follow_opponent && settings.motion() && !visible {
                pan_to = Some(p);
            }
        }
        targets.push((sighting.hex, alpha));
    }

    egui::Area::new(egui::Id::new("offscreen_pointers"))
        .order(egui::Order::Foreground)
        .interactable(true)
        .show(ctx, |ui| {
            let painter = ui.painter().with_clip_rect(board);
            let mut drawn: Vec<egui::Pos2> = Vec::new();
            for &(hex, alpha) in &targets {
                let p = world(hex);
                let Some(target) = on_screen(p) else { continue };
                let Some((anchor, dir)) = edge_anchor(board, target, 18.0) else {
                    continue;
                };
                // Several hexes past the same stretch of edge: one pointer.
                if drawn.iter().any(|d| d.distance(anchor) < 24.0) {
                    continue;
                }
                drawn.push(anchor);
                // A small chevron on a dark dot, pointing out of the board.
                let ink = crate::ui::palette::INK.gamma_multiply(0.7 * alpha);
                let light = egui::Color32::from_rgb(245, 232, 190).gamma_multiply(0.9 * alpha);
                let side = egui::vec2(-dir.y, dir.x);
                painter.circle_filled(anchor, 8.0, ink);
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        anchor + dir * 5.0,
                        anchor - dir * 3.0 + side * 4.5,
                        anchor - dir * 3.0 - side * 4.5,
                    ],
                    light,
                    egui::Stroke::NONE,
                ));
                let response = ui
                    .interact(
                        egui::Rect::from_center_size(anchor, egui::vec2(22.0, 22.0)),
                        egui::Id::new(("offscreen_pointer", hex.q, hex.r)),
                        egui::Sense::click(),
                    )
                    .on_hover_text(format!("{hex} — click to look"));
                if response.clicked() {
                    pan_to = Some(p);
                }
            }
        });

    if let Some(p) = pan_to {
        cam_state.focus.x = p.x;
        cam_state.focus.z = p.z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pointer_sits_on_the_edge_toward_its_target() {
        let board = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(200.0, 100.0));
        assert!(
            edge_anchor(board, egui::pos2(50.0, 50.0), 10.0).is_none(),
            "in view: no pointer"
        );
        let (at, dir) = edge_anchor(board, egui::pos2(500.0, 50.0), 10.0).unwrap();
        assert!((at.x - 190.0).abs() < 1e-3 && (at.y - 50.0).abs() < 1e-3);
        assert!(dir.x > 0.99);
        let (at, dir) = edge_anchor(board, egui::pos2(100.0, -400.0), 10.0).unwrap();
        assert!((at.y - 10.0).abs() < 1e-3);
        assert!(dir.y < -0.99);
    }

    #[test]
    fn only_the_latest_sightings_are_kept() {
        let mut sightings = Sightings::default();
        for q in 0..5 {
            sightings.see(HexCoord::new(q, 0));
        }
        let kept: Vec<i32> = sightings.0.iter().map(|s| s.hex.q).collect();
        assert_eq!(kept, vec![2, 3, 4]);
        // Seeing a hex again moves it to the front, not a second entry.
        sightings.see(HexCoord::new(3, 0));
        let kept: Vec<i32> = sightings.0.iter().map(|s| s.hex.q).collect();
        assert_eq!(kept, vec![2, 4, 3]);
    }
}

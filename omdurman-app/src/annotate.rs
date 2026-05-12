use bevy::prelude::*;
use omdurman_hex::{HexLayout, world_to_pixel};
use omdurman_types::HexCoord;

use crate::RtsCamera;
use crate::util::raycast_ground;

#[derive(Resource, Default)]
pub struct AnnotationSession {
    pub active: bool,
    pub points: Vec<AnnotationPoint>,
}

#[derive(Clone, Debug)]
pub struct AnnotationPoint {
    pub hex: HexCoord,
    pub pixel: Vec2,
}

/// Tab toggles annotation mode; left-click records a hex + pixel pair.
pub fn toggle_annotation_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<AnnotationSession>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }
    session.active = !session.active;
    if session.active {
        info!("Annotation mode ACTIVE — click hexes to record them. Tab to exit.");
    } else {
        print_results(&session);
        session.points.clear();
    }
}

pub fn handle_annotation_click(
    buttons: Res<ButtonInput<MouseButton>>,
    mut session: ResMut<AnnotationSession>,
    layout: Res<HexLayout>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
) {
    if !session.active || !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(world_pos) = raycast_ground(&windows, &cameras) else { return };
    let hex = layout.world_to_hex(world_pos);
    let pixel = world_to_pixel(world_pos);
    info!(
        "Annotated: ({:>3}, {:>3})  px ({:>8.0}, {:>8.0})",
        hex.q, hex.r, pixel.x, pixel.y
    );
    session.points.push(AnnotationPoint { hex, pixel });
}

fn print_results(session: &AnnotationSession) {
    info!("═══════════════════════════════════════════════════");
    info!("  Annotation Results — {} point(s)", session.points.len());
    info!("  {:<14} {:<12} {:<12}", "Hex (q, r)", "Pixel X", "Pixel Y");
    info!("  ─────────────────────────────────────────────────");
    for pt in &session.points {
        info!(
            "  ({:>3}, {:>3})    {:>8.0}    {:>8.0}",
            pt.hex.q, pt.hex.r, pt.pixel.x, pt.pixel.y
        );
    }
    info!("═══════════════════════════════════════════════════");
}

//! Peer cursor rendering over the board.
use super::*;

pub(crate) fn cursor_overlay_ui(
    mut contexts: EguiContexts,
    time: Res<Time>,
    local: Res<settings::LocalPlayerSettings>,
    local_peer: Res<LocalPeer>,
    mut peers: crate::peers::PeerCursorQuery,
    cameras: Query<(&Camera, &GlobalTransform), With<RtsCamera>>,
) {
    if !local.show_other_cursors {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let Ok((camera, cam_transform)) = cameras.single() else {
        return;
    };

    let now = time.elapsed_secs_f64();
    let dt = time.delta_secs();

    const SMOOTH: f32 = 6.0;
    let alpha = 1.0 - (-SMOOTH * dt).exp();

    let mut visible: Vec<(PeerId, Vec2, egui::Color32, String)> = Vec::new();
    for (entity, key, name, color, mut cursor) in &mut peers {
        // Skip the local player's own cursor (that's the mouse pointer).
        if local_peer.0 == Some(entity) {
            continue;
        }
        let Some(pos) = cursor.current else {
            continue;
        };
        let t = if cursor.last_update > 0.0 {
            let elapsed = now - cursor.last_update;
            (elapsed / 0.1).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let prev = cursor.previous.unwrap_or(pos);
        let target = prev.lerp(pos, t as f32);
        let display = cursor.display.get_or_insert(target);
        *display = display.lerp(target, alpha);

        let color = color.map(|c| c.0).unwrap_or(egui::Color32::WHITE);
        let name = name
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("{:?}", key.0));
        visible.push((key.0, *display, color, name));
    }

    if visible.is_empty() {
        return;
    }

    egui::Area::new(egui::Id::new("cursor_overlay"))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            for (_, world_xz, color, label) in &visible {
                let world = Vec3::new(world_xz.x, 0.0, world_xz.y);
                let Ok(viewport) = camera.world_to_viewport(cam_transform, world) else {
                    continue;
                };
                let screen = egui::pos2(viewport.x, viewport.y);
                painter.circle_filled(screen, 5.0, *color);
                painter.text(
                    screen + egui::Vec2::new(8.0, -4.0),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(12.0),
                    *color,
                );
            }
        });
}

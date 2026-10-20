//! Shared board-overlay scaffolding: what every hex-overlay system works
//! with -- the commands that spawn and despawn its rings and arrows, the
//! board geometry they are laid out in, and whether what it drew last is
//! still there and still in place.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use omdurman_hexmap::hex_world_pos;
use omdurman_types::{HexCoord, OverlayParams};

use crate::params::HexRender;
use crate::picker::OverlayGeneration;
use crate::render::MovementArrowAssets;

/// A board-overlay system's working set. Each ring and arrow is laid out at
/// this frame's board geometry.
#[derive(SystemParam)]
pub(crate) struct Overlay<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub hex: HexRender<'w>,
    pub arrow_assets: Res<'w, MovementArrowAssets>,
    generation: Res<'w, OverlayGeneration>,
    /// The overlay generation this system last saw.
    seen: Local<'s, u32>,
}

impl Overlay<'_, '_> {
    /// Whether what the system drew last is gone or misplaced: the overlays
    /// were cleared (a round trip through the menu or lobby) or the board
    /// geometry changed (a board was loaded) since it last looked. A system
    /// that caches what it drew forgets it then and draws afresh.
    pub(crate) fn stale(&mut self) -> bool {
        let cleared = self.generation.invalidates(&mut self.seen);
        cleared || self.hex.geometry_changed()
    }

    /// Despawn the previous overlay entities.
    pub(crate) fn clear(&mut self, existing: impl IntoIterator<Item = Entity>) {
        for e in existing {
            self.commands.entity(e).despawn();
        }
    }

    /// The board's adjusted origin, for spawns outside the plain ring shape
    /// (bars, labels).
    pub(crate) fn origin(&self) -> Vec2 {
        self.hex.layout.adjusted_origin(&self.hex.overlay.params)
    }

    /// The hex size.
    pub(crate) fn size(&self) -> f32 {
        self.hex.overlay.params.hex_size
    }

    /// The overlay params, for spawns needing full layout math.
    pub(crate) fn params(&self) -> &OverlayParams {
        &self.hex.overlay.params
    }

    /// Spawn one hex ring at `coord`, `height` above the plane, scaled by
    /// `scale` (1.0 = the full hex outline) and tinted with `material`.
    pub(crate) fn ring(
        &mut self,
        marker: impl Component,
        coord: HexCoord,
        height: f32,
        scale: f32,
        material: Handle<StandardMaterial>,
    ) {
        let pos = hex_world_pos(coord, self.origin(), self.params());
        let transform =
            Transform::from_xyz(pos.x, height, pos.z).with_scale(Vec3::splat(self.size() * scale));
        let mesh = self.hex.assets.mesh.clone();
        self.commands.spawn((
            marker,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            Visibility::Visible,
        ));
    }

    /// Spawn a combat arrow from hex `from` to hex `to`, inset from both
    /// centres so it reads between the two counters.
    pub(crate) fn arrow(&mut self, from: HexCoord, to: HexCoord, marker: impl Component) {
        let (origin, params, size) = (self.origin(), self.params(), self.size());
        let from = hex_world_pos(from, origin, params);
        let to = hex_world_pos(to, origin, params);
        let delta = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
        let len = delta.length();
        if len < f32::EPSILON {
            return;
        }
        let dir = delta / len;
        let inset = size * 0.18;
        let draw_len = (len - inset).max(len * 0.4);
        let tail = from + dir * ((len - draw_len) * 0.5);
        let mesh = self.arrow_assets.mesh.clone();
        let material = self.arrow_assets.combat.clone();
        self.commands.spawn((
            marker,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(tail.x, 1.55, tail.z)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir))
                .with_scale(Vec3::new(size * 0.5, 1.0, draw_len)),
            Visibility::Visible,
        ));
    }
}

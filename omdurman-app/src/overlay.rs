//! Shared board-overlay scaffolding: the despawn-then-respawn ring batch
//! every hex-overlay system builds. One constructor replaces the repeated
//! destructure/despawn/geometry prologue (and keeps the despawn happening
//! before any early return, as the hand-rolled version did).

use bevy::prelude::*;
use omdurman_hexmap::hex_world_pos;
use omdurman_types::{HexCoord, OverlayParams};

use crate::params::HexRender;

/// A fresh overlay batch: the previous entities are queued for despawn, and
/// each [`ring`](RingBatch::ring) call spawns one hex ring (shared hex-ring
/// mesh) at this frame's board geometry.
pub(crate) struct RingBatch {
    mesh: Handle<Mesh>,
    origin: Vec2,
    params: OverlayParams,
}

impl RingBatch {
    /// Spawn one hex ring at `coord`, `height` above the plane, scaled by
    /// `scale` (1.0 = the full hex outline) and tinted with `material`.
    pub(crate) fn ring(
        &self,
        commands: &mut Commands,
        marker: impl Component,
        coord: HexCoord,
        height: f32,
        scale: f32,
        material: &Handle<StandardMaterial>,
    ) {
        let pos = hex_world_pos(coord, self.origin, &self.params);
        commands.spawn((
            marker,
            Mesh3d(self.mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(pos.x, height, pos.z)
                .with_scale(Vec3::splat(self.params.hex_size * scale)),
            Visibility::Visible,
        ));
    }

    /// The batch's adjusted board origin, for spawns outside the plain
    /// ring shape (bars, arrows, labels).
    pub(crate) fn origin(&self) -> Vec2 {
        self.origin
    }

    /// The batch's hex size.
    pub(crate) fn size(&self) -> f32 {
        self.params.hex_size
    }

    /// The batch's overlay params, for spawns needing full layout math.
    pub(crate) fn params(&self) -> &OverlayParams {
        &self.params
    }
}

/// Despawn the previous overlay batch and return a builder for the new one.
pub(crate) fn ring_batch(
    commands: &mut Commands,
    hex: &HexRender,
    existing: impl IntoIterator<Item = Entity>,
) -> RingBatch {
    for e in existing {
        commands.entity(e).despawn();
    }
    RingBatch {
        mesh: hex.assets.mesh.clone(),
        origin: hex.layout.adjusted_origin(&hex.overlay.params),
        params: hex.overlay.params.clone(),
    }
}
